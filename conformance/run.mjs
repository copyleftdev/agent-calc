// Three transports, one kernel — prove it.
//
// The CLI, the WebAssembly build and calcd all call `execute_tool_json`. That is
// the design, but a design is not evidence: they are compiled separately, run on
// different targets, and reach the function through different serialisation. The
// claim that matters to anything built on this — run it again and you get the
// same answer — is a claim about all three agreeing, so it has to be measured.
//
//   node conformance/run.mjs            CLI + wasm + calcd (if it is up)
//   CALCD=http://host:8787 node …       point at a running server
//
// The corpus is not invented. It is every request shape bento actually issues,
// captured from its four worlds, because the shapes a real caller sends are the
// ones worth guaranteeing.

import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const corpus = JSON.parse(readFileSync(resolve(here, 'corpus.json'), 'utf8'));
const CALCD = process.env.CALCD ?? 'http://127.0.0.1:8787';
const CLI = process.env.CLI ?? resolve(here, '..', 'target', 'release', 'agent-calc');
const WASM_DIR = process.env.WASM_DIR ?? '/home/ops/Project/bento-budget/vendor/wasm';

// Key order is not meaning. Two responses that differ only in how a serialiser
// felt about ordering are the same answer, and a conformance run that reported
// otherwise would cry wolf until nobody ran it.
function canon(v) {
  if (Array.isArray(v)) return v.map(canon);
  if (v && typeof v === 'object') {
    return Object.fromEntries(Object.keys(v).sort().map((k) => [k, canon(v[k])]));
  }
  return v;
}
const same = (a, b) => JSON.stringify(canon(a)) === JSON.stringify(canon(b));

// Two tiers, because the two kinds of difference are not the same kind of
// problem.
//
// MUST: nothing that changes a decision or a displayed figure. Feasible vs
// infeasible, a survivor count, a dollar. A failure here is a bug and blocks.
//
// NOTE: the last bit or two of an f64, below anything a caller can act on.
// Measured in the product: routing a 121-cell survival map through the server
// instead of the browser moved 13 cells in the fifteenth significant figure and
// changed no feasibility, no survivor count, and no edge. Blocking on that would
// mean a red build that nobody can fix and everybody learns to ignore, which is
// worse than a build that reports it honestly.
const TOL = 1e-9;

function compare(a, b, path = '', hard = [], soft = []) {
  if (typeof a === 'number' && typeof b === 'number') {
    if (a === b || (Number.isNaN(a) && Number.isNaN(b))) return { hard, soft };
    const rel = Math.abs(a - b) / Math.max(1, Math.abs(a), Math.abs(b));
    (rel <= TOL ? soft : hard).push({ path, a, b, rel });
    return { hard, soft };
  }
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) { hard.push({ path, a: `len ${a.length}`, b: `len ${b.length}` }); return { hard, soft }; }
    a.forEach((_, i) => compare(a[i], b[i], `${path}[${i}]`, hard, soft));
    return { hard, soft };
  }
  if (a && b && typeof a === 'object' && typeof b === 'object') {
    const keys = new Set([...Object.keys(a), ...Object.keys(b)]);
    for (const k of keys) compare(a[k], b[k], path ? `${path}.${k}` : k, hard, soft);
    return { hard, soft };
  }
  if (!same(a, b)) hard.push({ path, a, b });
  return { hard, soft };
}

// The wasm bridge and calcd both return the tool envelope; the CLI prints the
// bare domain response. Comparing the inner payload is comparing the arithmetic,
// which is the thing under test.
const inner = (r) => (r && typeof r === 'object' && 'output' in r ? r.output : r);

// ---- transport 1: the CLI, one process per call ----------------------------
function viaCli(c) {
  const out = execFileSync(CLI, [c.command], {
    input: JSON.stringify(c.request),
    maxBuffer: 32 * 1024 * 1024,
  }).toString();
  return JSON.parse(out);
}

// ---- transport 2: WebAssembly, in this process ------------------------------
async function loadWasm() {
  const glue = resolve(WASM_DIR, 'agent_calc.js');
  const mod = await import(glue);
  const bytes = readFileSync(resolve(WASM_DIR, 'agent_calc_bg.wasm'));
  // wasm-pack's web target expects to fetch its own bytes; in node it is handed
  // them directly instead.
  await mod.default({ module_or_path: bytes });
  return (c) => JSON.parse(mod.execute_oddly_exact(c.command, JSON.stringify(c.request)));
}

// ---- transport 3: calcd, everything in one request --------------------------
async function viaServer(cases) {
  const r = await fetch(`${CALCD}/v1/batch`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ calls: cases.map((c) => ({ command: c.command, request: c.request })) }),
  });
  if (!r.ok) throw new Error(`calcd ${r.status}`);
  const body = await r.json();
  if (body.count !== cases.length) throw new Error(`calcd returned ${body.count} of ${cases.length}`);
  return { responses: body.responses, elapsed_ms: body.elapsed_ms };
}

// ---- run --------------------------------------------------------------------
const results = { cli: [], wasm: [], server: null };
let wasmRun = null;
try {
  wasmRun = await loadWasm();
} catch (e) {
  console.log(`  wasm unavailable, skipping that transport — ${String(e.message).slice(0, 90)}`);
}

const t0 = Date.now();
for (const c of corpus) results.cli.push(viaCli(c));
const cliMs = Date.now() - t0;

if (wasmRun) {
  const t = Date.now();
  for (const c of corpus) results.wasm.push(wasmRun(c));
  var wasmMs = Date.now() - t;
}

let serverMs = null;
try {
  const t = Date.now();
  results.server = await viaServer(corpus);
  serverMs = Date.now() - t;
} catch (e) {
  console.log(`  calcd unavailable, skipping that transport — ${String(e.message).slice(0, 90)}`);
}

let failed = 0;
let noted = 0;
const notedCases = [];
for (let i = 0; i < corpus.length; i += 1) {
  const c = corpus[i];
  const base = inner(results.cli[i]);
  const pairs = [];
  if (wasmRun) pairs.push(['wasm', inner(results.wasm[i])]);
  if (results.server) pairs.push(['calcd', inner(results.server.responses[i])]);

  for (const [what, got] of pairs) {
    const { hard, soft } = compare(base, got);
    if (hard.length) {
      failed += 1;
      console.log(`\n  FAIL  ${c.id}  (${c.command})  cli vs ${what}`);
      for (const d of hard.slice(0, 3)) console.log(`    ${d.path}: ${d.a}  vs  ${d.b}`);
    }
    if (soft.length) {
      noted += soft.length;
      if (notedCases.length < 5) notedCases.push(`${c.id} ${what} ${soft[0].path} (${soft[0].rel.toExponential(1)} rel)`);
    }
  }
}

const transports = 1 + (wasmRun ? 1 : 0) + (results.server ? 1 : 0);
console.log(`\n  ${corpus.length} cases x ${transports} transports`);
console.log(`    cli    ${cliMs} ms   (one process per call)`);
if (wasmRun) console.log(`    wasm   ${wasmMs} ms   (in this process)`);
if (results.server) {
  console.log(`    calcd  ${serverMs} ms   (one request; ${results.server.elapsed_ms.toFixed(1)} ms of it compute)`);
}
if (noted) {
  console.log(`\n  ${noted} value(s) differ below ${TOL} relative — recorded, not blocking:`);
  for (const n of notedCases) console.log(`    ${n}`);
  console.log(`    (nothing here changes a decision or a displayed figure)`);
}
console.log(failed ? `\n  ${failed} case(s) FAILED` : `\n  no case differs in anything a caller can act on`);
process.exit(failed ? 1 : 0);
