import initCalculator, {
  execute_oddly_exact,
  schema_oddly_exact,
} from "@/public/wasm/agent_calc.js";

import type {
  CalculatorCommand,
  CalculatorEnvelope,
} from "./types";

let calculatorReady: Promise<unknown> | undefined;

async function ensureCalculator(requestUrl: string): Promise<void> {
  if (!calculatorReady) {
    const wasmUrl = new URL("/wasm/agent_calc_bg.wasm", requestUrl);
    calculatorReady = initCalculator({
      module_or_path: fetch(wasmUrl, {
        headers: { Accept: "application/wasm" },
      }),
    });
  }

  await calculatorReady;
}

export async function calculatorSchema(
  command: CalculatorCommand,
  requestUrl: string,
): Promise<Record<string, unknown>> {
  await ensureCalculator(requestUrl);
  return JSON.parse(schema_oddly_exact(command)) as Record<string, unknown>;
}

export async function executeCalculator(
  command: CalculatorCommand,
  request: Record<string, unknown>,
  requestUrl: string,
): Promise<CalculatorEnvelope> {
  await ensureCalculator(requestUrl);
  const result = execute_oddly_exact(command, JSON.stringify(request));
  return JSON.parse(result) as CalculatorEnvelope;
}
