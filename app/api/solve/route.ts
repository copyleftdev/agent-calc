import {
  calculatorSchema,
  executeCalculator,
} from "@/lib/oddly-exact/calculator";
import { canonicalizeToolRequest } from "@/lib/oddly-exact/canonicalize";
import {
  classifyQuestion,
  compileToolRequest,
  explainEvidence,
  openRouterErrorStatus,
  selectToolSchema,
} from "@/lib/oddly-exact/openrouter";
import {
  assertSameOrigin,
  enforceLocalRateLimit,
  GuardrailError,
  narrationIsGrounded,
  normalizeQuestion,
  safeFallbackNarration,
  ungroundedNarrationNumbers,
} from "@/lib/oddly-exact/security";
import {
  QuestionSchema,
  type CalculatorCommand,
  type SolveEvent,
} from "@/lib/oddly-exact/types";

export const dynamic = "force-dynamic";
export const maxDuration = 45;

function clientKey(request: Request): string {
  return (
    request.headers.get("cf-connecting-ip") ||
    request.headers.get("x-forwarded-for")?.split(",")[0]?.trim() ||
    "local"
  );
}

function errorEvent(error: unknown): SolveEvent {
  if (error instanceof GuardrailError) {
    return {
      type: "error",
      at: Date.now(),
      data: { code: error.code, reason: error.message, retryable: false },
    };
  }

  const status = openRouterErrorStatus(error);
  const reason =
    status === 401 || status === 403
      ? "The interpretation service is not authorized."
      : status === 402
        ? "The interpretation service has insufficient credits."
        : status === 408 || status === 429 || status >= 500
          ? "The interpretation service is temporarily unavailable."
          : "The proof could not be assembled from the calculator evidence.";

  return {
    type: "error",
    at: Date.now(),
    data: {
      code: status === 429 ? "rate_limited" : "interpretation_failed",
      reason,
      retryable: status === 408 || status === 429 || status >= 500,
    },
  };
}

export async function POST(request: Request): Promise<Response> {
  if (!request.headers.get("content-type")?.includes("application/json")) {
    return Response.json(
      { error: { code: "invalid_content_type", reason: "Expected JSON." } },
      { status: 415 },
    );
  }

  const encoder = new TextEncoder();

  return new Response(
    new ReadableStream({
      async start(controller) {
        const send = (event: SolveEvent) => {
          controller.enqueue(encoder.encode(`${JSON.stringify(event)}\n`));
        };

        try {
          assertSameOrigin(request);
          enforceLocalRateLimit(clientKey(request));

          const body = QuestionSchema.parse(await request.json());
          const question = normalizeQuestion(body.question);
          send({
            type: "accepted",
            at: Date.now(),
            data: { question },
          });

          const classification = await classifyQuestion(question);
          send({
            type: "interpreted",
            at: Date.now(),
            data: classification,
          });

          if (classification.command === "unsupported") {
            send({
              type: "refused",
              at: Date.now(),
              data: {
                code: "unsupported_question",
                reason:
                  "This question could not be represented by one supported Oddly Exact operation.",
              },
            });
            return;
          }

          const command = classification.command as CalculatorCommand;
          const fullSchema = await calculatorSchema(command, request.url);
          const schema = await selectToolSchema(
            question,
            classification,
            fullSchema,
          );
          let compiled = await compileToolRequest(
            question,
            classification,
            schema,
          );
          compiled.request = canonicalizeToolRequest(compiled.request);
          if (process.env.ODDLY_EXACT_DEBUG === "1") {
            console.error(
              "[oddly-exact] canonical tool request",
              JSON.stringify(compiled.request),
            );
          }
          let calculatorEvidence = await executeCalculator(
            command,
            compiled.request,
            request.url,
          );
          let validationRetries = 0;

          if (
            calculatorEvidence.status === "rejected" &&
            calculatorEvidence.error?.code === "invalid_request"
          ) {
            compiled = await compileToolRequest(
              question,
              classification,
              schema,
              {
                assistantMessage: compiled.assistantMessage,
                toolCallId: compiled.toolCallId,
                calculatorRejection: calculatorEvidence,
              },
            );
            compiled.request = canonicalizeToolRequest(compiled.request);
            if (process.env.ODDLY_EXACT_DEBUG === "1") {
              console.error(
                "[oddly-exact] corrected canonical tool request",
                JSON.stringify(compiled.request),
              );
            }
            calculatorEvidence = await executeCalculator(
              command,
              compiled.request,
              request.url,
            );
            validationRetries = 1;
          }

          send({
            type: "tool_call",
            at: Date.now(),
            data: {
              command,
              request: compiled.request,
              validation_retries: validationRetries,
            },
          });

          send({
            type: "tool_result",
            at: Date.now(),
            data: calculatorEvidence,
          });

          const evidenceBytes = JSON.stringify(calculatorEvidence).length;
          if (evidenceBytes > 64 * 1024) {
            throw new GuardrailError(
              "evidence_too_large",
              "The calculator result is too large to narrate safely. Narrow the request.",
            );
          }

          let proposedNarration = await explainEvidence(
            question,
            classification,
            command,
            compiled.request,
            calculatorEvidence,
          );
          const firstPassUngrounded = ungroundedNarrationNumbers(
            proposedNarration,
            question,
            calculatorEvidence,
          );
          if (firstPassUngrounded.length > 0) {
            proposedNarration = await explainEvidence(
              question,
              classification,
              command,
              compiled.request,
              calculatorEvidence,
              { forbiddenNumbers: [...new Set(firstPassUngrounded)] },
            );
          }
          const narration = narrationIsGrounded(
            proposedNarration,
            question,
            calculatorEvidence,
          )
            ? proposedNarration
            : safeFallbackNarration(calculatorEvidence);
          if (
            process.env.ODDLY_EXACT_DEBUG === "1" &&
            narration !== proposedNarration
          ) {
            console.error(
              "[oddly-exact] withheld narration numbers",
              ungroundedNarrationNumbers(
                proposedNarration,
                question,
                calculatorEvidence,
              ),
              proposedNarration,
            );
          }

          send({ type: "explained", at: Date.now(), data: narration });
          send({
            type: "complete",
            at: Date.now(),
            data: { grounded: true },
          });
        } catch (error) {
          if (process.env.ODDLY_EXACT_DEBUG === "1") {
            console.error("[oddly-exact] solve pipeline failure", error);
          }
          send(errorEvent(error));
        } finally {
          controller.close();
        }
      },
    }),
    {
      headers: {
        "Content-Type": "application/x-ndjson; charset=utf-8",
        "Cache-Control": "no-store",
        "X-Content-Type-Options": "nosniff",
      },
    },
  );
}
