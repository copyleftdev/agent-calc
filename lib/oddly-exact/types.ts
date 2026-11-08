import { z } from "zod";

export const CALCULATOR_COMMANDS = [
  "eval",
  "simplify",
  "trace",
  "substitute",
  "assumptions",
  "solve",
  "calculus",
  "inequality",
  "polynomial",
  "interval",
  "finance",
  "units",
  "matrix",
  "stats",
  "optimize",
  "linear",
  "complex",
  "number",
] as const;

export type CalculatorCommand = (typeof CALCULATOR_COMMANDS)[number];

export const QuestionSchema = z
  .object({
    question: z.string().trim().min(3).max(1600),
  })
  .strict();

export const ClassificationSchema = z
  .object({
    command: z.enum([...CALCULATOR_COMMANDS, "unsupported"]),
    interpretation: z.string().trim().min(1).max(360),
    rationale: z.string().trim().min(1).max(360),
    assumptions: z.array(z.string().trim().min(1).max(220)).max(6),
  })
  .strict();

export type Classification = z.infer<typeof ClassificationSchema>;

export const NarrationSchema = z
  .object({
    display_formula: z.string().max(320),
    answer: z.string().trim().min(1).max(720),
    method: z.string().trim().min(1).max(480),
    steps: z
      .array(
        z
          .object({
            title: z.string().trim().min(1).max(100),
            explanation: z.string().trim().min(1).max(520),
            evidence_paths: z.array(z.string().max(180)).max(5),
          })
          .strict(),
      )
      .min(1)
      .max(7),
    caveats: z.array(z.string().trim().min(1).max(320)).max(5),
  })
  .strict();

export type Narration = z.infer<typeof NarrationSchema>;

export type CalculatorEnvelope = {
  status: "executed" | "rejected" | "failed";
  contract_version?: string;
  command?: string;
  output?: unknown;
  error?: {
    code: string;
    reason: string;
  };
};

export type SolveEvent =
  | { type: "accepted"; at: number; data: { question: string } }
  | { type: "interpreted"; at: number; data: Classification }
  | {
      type: "tool_call";
      at: number;
      data: {
        command: CalculatorCommand;
        request: Record<string, unknown>;
        validation_retries?: number;
      };
    }
  | { type: "tool_result"; at: number; data: CalculatorEnvelope }
  | { type: "explained"; at: number; data: Narration }
  | {
      type: "refused";
      at: number;
      data: { code: string; reason: string };
    }
  | { type: "complete"; at: number; data: { grounded: true } }
  | {
      type: "error";
      at: number;
      data: { code: string; reason: string; retryable: boolean };
    };
