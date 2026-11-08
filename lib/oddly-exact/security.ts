import type { Narration } from "./types";

const INJECTION_PATTERNS = [
  /\bignore\s+(all\s+)?(previous|prior|system|developer)\b/i,
  /\b(reveal|repeat|print|show)\s+(the\s+)?(system|developer)\s+(prompt|message|instructions?)\b/i,
  /\bdo\s+not\s+(use|call|invoke)\s+(the\s+)?(tool|calculator)\b/i,
  /\b(jailbreak|prompt\s*injection|developer\s*mode|dan\s+mode)\b/i,
  /\b(exfiltrate|leak)\b.{0,24}\b(secret|token|key|prompt)\b/i,
  /<\s*\/?\s*(system|developer|tool|assistant)\b/i,
];

const CONTROL_CHARACTERS = /[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F]/g;
const NUMBER_TOKEN =
  /[-+]?(?:\d{1,3}(?:,\d{3})+|\d+)(?:\.\d+)?(?:e[-+]?\d+)?(?:\/\d+)?/gi;

export class GuardrailError extends Error {
  constructor(
    readonly code: string,
    message: string,
  ) {
    super(message);
  }
}

export function normalizeQuestion(question: string): string {
  const normalized = question
    .normalize("NFKC")
    .replace(CONTROL_CHARACTERS, "")
    .trim();

  if (INJECTION_PATTERNS.some((pattern) => pattern.test(normalized))) {
    throw new GuardrailError(
      "prompt_injection",
      "This request contains instructions aimed at changing the calculator boundary. Rephrase it as a mathematical question only.",
    );
  }

  const urlCount = (normalized.match(/https?:\/\//gi) ?? []).length;
  if (urlCount > 1) {
    throw new GuardrailError(
      "external_content",
      "Oddly Exact does not retrieve or follow external instructions. Paste the mathematical quantities directly.",
    );
  }

  return normalized;
}

function numericTokens(value: unknown): string[] {
  const serialized =
    typeof value === "string" ? value : JSON.stringify(value) ?? "";
  return serialized.match(NUMBER_TOKEN) ?? [];
}

function normalizeNumberToken(token: string): string {
  return token.toLowerCase().replaceAll(",", "").replace("−", "-");
}

export function narrationIsGrounded(
  narration: Narration,
  question: string,
  calculatorEvidence: unknown,
): boolean {
  return ungroundedNarrationNumbers(
    narration,
    question,
    calculatorEvidence,
  ).length === 0;
}

export function ungroundedNarrationNumbers(
  narration: Narration,
  question: string,
  calculatorEvidence: unknown,
): string[] {
  const allowed = new Set(
    numericTokens({ question, calculatorEvidence }).map(normalizeNumberToken),
  );
  const narrativeClaims = {
    display_formula: narration.display_formula.replace(
      /\^\{?[-+]?\d+\}?/g,
      "^",
    ),
    answer: narration.answer,
    method: narration.method,
    steps: narration.steps.map(({ title, explanation }) => ({
      title,
      explanation,
    })),
    caveats: narration.caveats,
  };

  return numericTokens(narrativeClaims)
    .map(normalizeNumberToken)
    .filter((token) => !allowed.has(token));
}

function evidenceExpressionToLatex(value: unknown): string {
  if (!value || typeof value !== "object") return "";
  const expression = value as Record<string, unknown>;
  const left = () => evidenceExpressionToLatex(expression.left);
  const right = () => evidenceExpressionToLatex(expression.right);
  const nested = () => evidenceExpressionToLatex(expression.value);

  switch (expression.kind) {
    case "integer":
      return typeof expression.value === "string" ? expression.value : "";
    case "rational":
      return `\\frac{${String(expression.numerator ?? "")}}{${String(expression.denominator ?? "")}}`;
    case "symbol":
      return typeof expression.name === "string" ? expression.name : "";
    case "add":
      return `${left()} + ${right()}`;
    case "sub":
      return `${left()} - ${right()}`;
    case "mul":
      return `${left()} \\cdot ${right()}`;
    case "div":
      return `\\frac{${left()}}{${right()}}`;
    case "pow":
      return `{${evidenceExpressionToLatex(expression.base)}}^{${String(expression.exponent ?? "")}}`;
    case "neg":
      return `-${nested()}`;
    case "sqrt":
      return `\\sqrt{${nested()}}`;
    case "sin":
    case "cos":
    case "tan":
    case "ln":
    case "exp":
      return `\\${String(expression.kind)}\\left(${nested()}\\right)`;
    case "abs":
      return `\\left|${nested()}\\right|`;
    default:
      return "";
  }
}

export function safeFallbackNarration(
  calculatorEvidence?: unknown,
): Narration {
  const envelope =
    calculatorEvidence && typeof calculatorEvidence === "object"
      ? (calculatorEvidence as Record<string, unknown>)
      : undefined;
  const output =
    envelope?.output && typeof envelope.output === "object"
      ? (envelope.output as Record<string, unknown>)
      : undefined;
  const displayFormula = evidenceExpressionToLatex(output?.expr);

  return {
    display_formula: displayFormula,
    answer:
      displayFormula
        ? "The deterministic calculator returned the exact symbolic result shown below."
        : "Oddly Exact returned structured evidence. Inspect the verified result below.",
    method:
      "The language model’s prose was withheld because it introduced a numerical claim that was not present in the calculator evidence.",
    steps: [
      {
        title: "Deterministic evidence preserved",
        explanation:
          "The raw calculator request and response remain available for inspection and copying.",
        evidence_paths: ["$"],
      },
    ],
    caveats: [
      "No ungrounded model-generated arithmetic was shown.",
    ],
  };
}

const rateBuckets = new Map<string, { count: number; resetsAt: number }>();

export function enforceLocalRateLimit(key: string, now = Date.now()): void {
  const current = rateBuckets.get(key);

  if (!current || current.resetsAt <= now) {
    rateBuckets.set(key, { count: 1, resetsAt: now + 60_000 });
    return;
  }

  if (current.count >= 8) {
    throw new GuardrailError(
      "rate_limited",
      "The proof engine is receiving too many requests from this connection. Try again shortly.",
    );
  }

  current.count += 1;
}

export function assertSameOrigin(request: Request): void {
  const origin = request.headers.get("origin");
  if (!origin) return;

  const forwardedHost = request.headers
    .get("x-forwarded-host")
    ?.split(",")[0]
    ?.trim();
  const requestHost =
    forwardedHost || request.headers.get("host") || new URL(request.url).host;

  if (new URL(origin).host !== requestHost) {
    throw new GuardrailError(
      "cross_origin_request",
      "Cross-origin solver requests are not accepted.",
    );
  }
}
