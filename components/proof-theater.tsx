"use client";

import * as Accordion from "@radix-ui/react-accordion";
import {
  ArrowDownIcon,
  ArrowRightIcon,
  CheckCircledIcon,
  ChevronDownIcon,
  CodeIcon,
  CopyIcon,
  CrossCircledIcon,
  CubeIcon,
  ExclamationTriangleIcon,
  GitHubLogoIcon,
  LightningBoltIcon,
  LockClosedIcon,
  MagicWandIcon,
  ReaderIcon,
  ReloadIcon,
  SewingPinFilledIcon,
  StopIcon,
} from "@radix-ui/react-icons";
import * as Tooltip from "@radix-ui/react-tooltip";
import {
  AnimatePresence,
  LayoutGroup,
  motion,
  useReducedMotion,
} from "motion/react";
import Link from "next/link";
import {
  FormEvent,
  useCallback,
  useMemo,
  useRef,
  useState,
} from "react";

import type {
  CalculatorEnvelope,
  Classification,
  Narration,
  SolveEvent,
} from "@/lib/oddly-exact/types";
import { Formula } from "./formula";

const EXAMPLES = [
  "Solve 3x + 7 = 28",
  "What are the mean and sample variance of 12, 15, 18, 21, 24?",
  "Differentiate 3x² + 2x − 5",
];

type ScalarLeaf = {
  path: string;
  value: string;
};

function flattenScalars(
  value: unknown,
  path = "$",
  leaves: ScalarLeaf[] = [],
): ScalarLeaf[] {
  if (leaves.length >= 10) return leaves;

  if (
    value === null ||
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean"
  ) {
    leaves.push({ path, value: String(value) });
    return leaves;
  }

  if (Array.isArray(value)) {
    value.slice(0, 5).forEach((item, index) => {
      flattenScalars(item, `${path}[${index}]`, leaves);
    });
    return leaves;
  }

  if (typeof value === "object") {
    Object.entries(value as Record<string, unknown>).forEach(([key, item]) => {
      flattenScalars(item, `${path}.${key}`, leaves);
    });
  }

  return leaves;
}

function stageMotion(reducedMotion: boolean | null) {
  return {
    initial: reducedMotion ? false : { opacity: 0, y: 22, scale: 0.99 },
    animate: { opacity: 1, y: 0, scale: 1 },
    transition: {
      duration: reducedMotion ? 0 : 0.52,
      ease: [0.16, 1, 0.3, 1] as [number, number, number, number],
    },
  };
}

function CopyButton({ value, label }: { value: string; label: string }) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    await navigator.clipboard.writeText(value);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
  };

  return (
    <Tooltip.Root>
      <Tooltip.Trigger asChild>
        <button className="iconButton" type="button" onClick={copy}>
          {copied ? <CheckCircledIcon /> : <CopyIcon />}
          <span className="srOnly">{copied ? "Copied" : label}</span>
        </button>
      </Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content className="tooltip" sideOffset={8}>
          {copied ? "Copied" : label}
          <Tooltip.Arrow className="tooltipArrow" />
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  );
}

function JsonDisclosure({
  label,
  value,
}: {
  label: string;
  value: unknown;
}) {
  const serialized = JSON.stringify(value, null, 2);

  return (
    <Accordion.Root type="single" collapsible className="jsonDisclosure">
      <Accordion.Item value="json">
        <Accordion.Header>
          <Accordion.Trigger className="jsonTrigger">
            <span>
              <CodeIcon />
              {label}
            </span>
            <ChevronDownIcon className="chevron" aria-hidden />
          </Accordion.Trigger>
        </Accordion.Header>
        <Accordion.Content className="jsonContent">
          <div className="codeToolbar">
            <span>JSON · exact contract</span>
            <CopyButton value={serialized} label={`Copy ${label}`} />
          </div>
          <pre>
            <code>{serialized}</code>
          </pre>
        </Accordion.Content>
      </Accordion.Item>
    </Accordion.Root>
  );
}

function Stage({
  number,
  tone,
  icon,
  eyebrow,
  title,
  children,
}: {
  number: string;
  tone: "blue" | "amber" | "green" | "sand" | "red";
  icon: React.ReactNode;
  eyebrow: string;
  title: string;
  children: React.ReactNode;
}) {
  const reducedMotion = useReducedMotion();

  return (
    <motion.section
      className={`proofStage proofStage--${tone}`}
      {...stageMotion(reducedMotion)}
      layout
    >
      <div className="stageMarker" aria-hidden>
        {icon}
      </div>
      <div className="stageBody">
        <header className="stageHeader">
          <div>
            <span className="stageEyebrow">
              {number} / {eyebrow}
            </span>
            <h2>{title}</h2>
          </div>
        </header>
        {children}
      </div>
    </motion.section>
  );
}

function WaitingStage({ label }: { label: string }) {
  return (
    <motion.div
      className="waitingStage"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
    >
      <ReloadIcon className="spinner" />
      <span>{label}</span>
    </motion.div>
  );
}

function InterpretationStage({
  data,
}: {
  data: Classification;
}) {
  return (
    <Stage
      number="01"
      tone="blue"
      icon={<ReaderIcon />}
      eyebrow="Interpretation"
      title={data.interpretation}
    >
      <p className="stageLead">{data.rationale}</p>
      <div className="decisionRow">
        <span className="domainBadge">
          <SewingPinFilledIcon />
          {data.command.replaceAll("_", " ")}
        </span>
        <span className="boundaryNote">
          Procedure selected by model · no calculation performed
        </span>
      </div>
      {data.assumptions.length > 0 && (
        <div className="assumptionField">
          <div className="subheading">
            <ExclamationTriangleIcon />
            Explicit assumptions
          </div>
          <ul>
            {data.assumptions.map((assumption) => (
              <li key={assumption}>{assumption}</li>
            ))}
          </ul>
        </div>
      )}
    </Stage>
  );
}

function ToolCallStage({
  command,
  request,
  validationRetries = 0,
}: {
  command: string;
  request: Record<string, unknown>;
  validationRetries?: number;
}) {
  return (
    <Stage
      number="02"
      tone="amber"
      icon={<MagicWandIcon />}
      eyebrow="Formalization"
      title="Natural language became a typed request."
    >
      <p className="stageLead">
        The selected domain supplied its own executable JSON Schema. The model
        could populate that contract, but it could not change it.
      </p>
      <div className="toolSignature">
        <span>oddly_exact</span>
        <ArrowRightIcon />
        <strong>{command}</strong>
      </div>
      {validationRetries > 0 && (
        <p className="boundaryNote">
          The first argument shape failed deterministic validation. The model
          received only that typed rejection and corrected the call.
        </p>
      )}
      <JsonDisclosure label="Inspect tool request" value={request} />
    </Stage>
  );
}

function EvidenceStage({ data }: { data: CalculatorEnvelope }) {
  const leaves = useMemo(() => flattenScalars(data.output), [data.output]);
  const executed = data.status === "executed";

  return (
    <Stage
      number="03"
      tone={executed ? "green" : "red"}
      icon={executed ? <CheckCircledIcon /> : <CrossCircledIcon />}
      eyebrow="Deterministic evidence"
      title={
        executed
          ? "The calculator answered under contract."
          : "The calculator refused the request."
      }
    >
      <div className="verificationBand">
        <div className="verificationSeal">
          <LockClosedIcon />
          <span>
            <strong>{data.contract_version ?? "calc1"}</strong>
            Rust · WebAssembly · model-independent
          </span>
        </div>
        <span className={`statusStamp statusStamp--${data.status}`}>
          {data.status}
        </span>
      </div>

      {executed && leaves.length > 0 && (
        <dl className="evidenceGrid">
          {leaves.map((leaf) => (
            <div key={`${leaf.path}-${leaf.value}`}>
              <dt>{leaf.path.replace("$.output.", "").replace("$.", "")}</dt>
              <dd>{leaf.value}</dd>
            </div>
          ))}
        </dl>
      )}

      {!executed && data.error && (
        <div className="refusalCopy">
          <strong>{data.error.code.replaceAll("_", " ")}</strong>
          <p>{data.error.reason}</p>
        </div>
      )}

      <JsonDisclosure label="Inspect calculator evidence" value={data} />
    </Stage>
  );
}

function NarrationStage({ data }: { data: Narration }) {
  return (
    <Stage
      number="04"
      tone="sand"
      icon={<LightningBoltIcon />}
      eyebrow="Explanation"
      title={data.answer}
    >
      <Formula expression={data.display_formula} />
      <p className="method">{data.method}</p>

      <ol className="explanationSteps">
        {data.steps.map((step, index) => (
          <li key={`${step.title}-${index}`}>
            <span>{String(index + 1).padStart(2, "0")}</span>
            <div>
              <h3>{step.title}</h3>
              <p>{step.explanation}</p>
              {step.evidence_paths.length > 0 && (
                <small>{step.evidence_paths.join(" · ")}</small>
              )}
            </div>
          </li>
        ))}
      </ol>

      {data.caveats.length > 0 && (
        <div className="caveatGroup">
          <span>Caveats</span>
          <ul>
            {data.caveats.map((caveat) => (
              <li key={caveat}>{caveat}</li>
            ))}
          </ul>
        </div>
      )}
    </Stage>
  );
}

export function ProofTheater() {
  const [question, setQuestion] = useState("");
  const [events, setEvents] = useState<SolveEvent[]>([]);
  const [busy, setBusy] = useState(false);
  const abortRef = useRef<AbortController | null>(null);
  const reducedMotion = useReducedMotion();

  const interpretation = events.find(
    (event) => event.type === "interpreted",
  )?.data;
  const toolCall = events.find((event) => event.type === "tool_call")?.data;
  const toolResult = events.find(
    (event) => event.type === "tool_result",
  )?.data;
  const narration = events.find((event) => event.type === "explained")?.data;
  const refusal = events.find((event) => event.type === "refused")?.data;
  const error = events.find((event) => event.type === "error")?.data;
  const hasRun = events.length > 0;

  const appendEvent = useCallback((event: SolveEvent) => {
    setEvents((current) => [...current, event]);
  }, []);

  const solve = async (event?: FormEvent) => {
    event?.preventDefault();
    const trimmed = question.trim();
    if (trimmed.length < 3 || busy) return;

    abortRef.current?.abort();
    const abort = new AbortController();
    abortRef.current = abort;
    setEvents([]);
    setBusy(true);

    try {
      const response = await fetch("/api/solve", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ question: trimmed }),
        signal: abort.signal,
      });

      if (!response.ok || !response.body) {
        throw new Error("The proof stream could not be opened.");
      }

      const reader = response.body.getReader();
      const decoder = new TextDecoder();
      let buffer = "";

      while (true) {
        const { done, value } = await reader.read();
        buffer += decoder.decode(value ?? new Uint8Array(), { stream: !done });
        const lines = buffer.split("\n");
        buffer = lines.pop() ?? "";

        for (const line of lines) {
          if (!line.trim()) continue;
          appendEvent(JSON.parse(line) as SolveEvent);
        }

        if (done) {
          if (buffer.trim()) appendEvent(JSON.parse(buffer) as SolveEvent);
          break;
        }
      }
    } catch (requestError) {
      if ((requestError as Error).name !== "AbortError") {
        appendEvent({
          type: "error",
          at: Date.now(),
          data: {
            code: "connection_failed",
            reason:
              "The proof stream was interrupted before the evidence arrived.",
            retryable: true,
          },
        });
      }
    } finally {
      setBusy(false);
      abortRef.current = null;
    }
  };

  const stop = () => {
    abortRef.current?.abort();
    setBusy(false);
  };

  const tryExample = (example: string) => {
    setQuestion(example);
    setEvents([]);
  };

  return (
    <Tooltip.Provider delayDuration={280}>
      <LayoutGroup>
        <main className={hasRun ? "site site--active" : "site"}>
          <header className="siteHeader">
            <Link className="brand" href="/" aria-label="Oddly Exact home">
              <span className="brandMark" aria-hidden>
                <span />
              </span>
              <span>Oddly Exact</span>
            </Link>
            <div className="headerBoundary">
              <LockClosedIcon />
              The model never does the math
            </div>
            <a
              className="githubLink"
              href="https://github.com/copyleftdev/agent-calc"
              target="_blank"
              rel="noreferrer"
            >
              <GitHubLogoIcon />
              <span>Source</span>
            </a>
          </header>

          <section className="hero">
            <motion.div
              className="heroCopy"
              initial={reducedMotion ? false : { opacity: 0, y: 18 }}
              animate={{ opacity: hasRun ? 0.72 : 1, y: 0 }}
              transition={{ duration: reducedMotion ? 0 : 0.62 }}
            >
              <span className="kicker">An instrument for mathematical truth</span>
              <h1>
                Ask the question.
                <br />
                <em>Watch the proof unfold.</em>
              </h1>
              <p>
                AI interprets your intent. Oddly Exact performs the computation.
                Every assumption, tool call, and result remains visible.
              </p>
            </motion.div>

            <motion.aside
              className="boundaryMap"
              initial={reducedMotion ? false : { opacity: 0, x: 24 }}
              animate={{ opacity: hasRun ? 0.6 : 1, x: 0 }}
              transition={{ duration: reducedMotion ? 0 : 0.62, delay: 0.08 }}
              aria-label="Trust boundary"
            >
              <div>
                <ReaderIcon />
                <span>
                  <small>Model</small>
                  interprets
                </span>
              </div>
              <ArrowDownIcon />
              <div>
                <CubeIcon />
                <span>
                  <small>Calculator</small>
                  computes
                </span>
              </div>
              <ArrowDownIcon />
              <div>
                <CheckCircledIcon />
                <span>
                  <small>Evidence</small>
                  explains
                </span>
              </div>
            </motion.aside>
          </section>

          <motion.section
            className="composerShell"
            layout
            transition={{ duration: reducedMotion ? 0 : 0.55, ease: [0.16, 1, 0.3, 1] }}
          >
            <form className="composer" onSubmit={solve}>
              <label htmlFor="math-question">
                {hasRun ? "Ask another question" : "What do you want to understand?"}
              </label>
              <div className="composerInput">
                <textarea
                  id="math-question"
                  value={question}
                  onChange={(event) => setQuestion(event.target.value)}
                  placeholder="e.g. Solve 3x + 7 = 28 and show me why"
                  rows={hasRun ? 2 : 3}
                  maxLength={1600}
                  disabled={busy}
                  onKeyDown={(event) => {
                    if (
                      event.key === "Enter" &&
                      !event.shiftKey &&
                      !event.nativeEvent.isComposing
                    ) {
                      event.preventDefault();
                      void solve();
                    }
                  }}
                />
                {busy ? (
                  <button
                    className="submitButton submitButton--stop"
                    type="button"
                    onClick={stop}
                  >
                    <StopIcon />
                    Stop
                  </button>
                ) : (
                  <button
                    className="submitButton"
                    type="submit"
                    disabled={question.trim().length < 3}
                  >
                    Unfold
                    <ArrowRightIcon />
                  </button>
                )}
              </div>
              <div className="composerMeta">
                <span>
                  <LockClosedIcon />
                  Prompt-isolated · schema-bound · numerically grounded
                </span>
                <span>{question.length} / 1600</span>
              </div>
            </form>

            {!hasRun && (
              <div className="examples" aria-label="Example questions">
                {EXAMPLES.map((example) => (
                  <button
                    key={example}
                    type="button"
                    onClick={() => tryExample(example)}
                  >
                    {example}
                    <ArrowRightIcon />
                  </button>
                ))}
              </div>
            )}
          </motion.section>

          <AnimatePresence mode="popLayout">
            {hasRun && (
              <motion.div
                className="proofWorkspace"
                initial={reducedMotion ? false : { opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
              >
                <div className="proofHeading">
                  <span>Live proof record</span>
                  <div className="proofRule" />
                  <span>{busy ? "assembling" : "recorded"}</span>
                </div>

                <div className="proofRail">
                  {interpretation && (
                    <InterpretationStage data={interpretation} />
                  )}
                  {busy && !interpretation && (
                    <WaitingStage label="Interpreting mathematical intent" />
                  )}

                  {toolCall && (
                    <ToolCallStage
                      command={toolCall.command}
                      request={toolCall.request}
                      validationRetries={toolCall.validation_retries}
                    />
                  )}
                  {busy && interpretation && !toolCall && !refusal && (
                    <WaitingStage label="Compiling the typed calculator request" />
                  )}

                  {toolResult && <EvidenceStage data={toolResult} />}
                  {busy && toolCall && !toolResult && (
                    <WaitingStage label="Executing Oddly Exact in WebAssembly" />
                  )}

                  {narration && <NarrationStage data={narration} />}
                  {busy && toolResult && !narration && (
                    <WaitingStage label="Grounding the explanation in evidence" />
                  )}
                </div>

                {(refusal || error) && (
                  <motion.section
                    className="terminalMessage"
                    {...stageMotion(reducedMotion)}
                  >
                    <CrossCircledIcon />
                    <div>
                      <span>{refusal?.code ?? error?.code}</span>
                      <h2>{refusal?.reason ?? error?.reason}</h2>
                      {error?.retryable && (
                        <button type="button" onClick={() => void solve()}>
                          Try again <ReloadIcon />
                        </button>
                      )}
                    </div>
                  </motion.section>
                )}
              </motion.div>
            )}
          </AnimatePresence>

          <footer>
            <span>Oddly Exact · calc1/0.1.0</span>
            <span>Proof over vibes. Trust must survive challenge.</span>
          </footer>
        </main>
      </LayoutGroup>
    </Tooltip.Provider>
  );
}
