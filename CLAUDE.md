# agent-calc — Claude Development Guide

## What this project is

A contract-first Rust CLI and library for deterministic, exact computation.
Every command reads typed JSON from stdin and writes typed JSON to stdout.
No prose. No approximation unless flagged. No silent failures.

The design goal: an AI agent calling this tool must be able to trust every
output field, reconstruct every step, and handle every failure with a stable
error code — without reading documentation.

---

## The three invariants you must never break

1. **Exact-first** — if a result can be represented as an exact rational
   (`numerator`/`denominator`), it must be. Float results require
   `"exactness": "approximate_f64"` in the response. Omitting this field on a
   float result is a contract violation.

2. **Typed failures** — every error path returns a JSON object with a stable
   `"code"` from `ErrorCode`. Never panic, never write to stdout on error (use
   stderr), never return an untyped string as an error.

3. **Bounded inputs** — every handler must enforce the runtime limits defined
   in the protocol (expression depth, node count, digit count, symbol length,
   exponent size, binding count, decimal precision). A request that exceeds a
   limit must return `ErrorCode::InputTooLarge`, not hang or OOM.

---

## Repository layout

```
src/
  lib.rs          — public API re-exports
  main.rs         — CLI dispatch (one arm per command)
  protocol.rs     — Describe contract, ErrorCode enum, shared types
  rational.rs     — Exact rational arithmetic core
  eval.rs         — Expression evaluator
  simplify.rs     — Symbolic simplifier
  trace.rs        — Proof-trace wrapper
  substitute.rs   — Symbol binding
  assumptions.rs  — Symbolic domain assumptions + entailment
  solve.rs        — Affine equation solver
  calculus.rs     — Symbolic differentiator
  inequality.rs   — Affine inequality solver
  polynomial.rs   — Exact rational polynomial ops
  interval.rs     — Exact rational interval arithmetic
  finance.rs      — Time-value-of-money
  units.rs        — Dimension-checked unit conversion (uom)
  matrix.rs       — f64 matrix ops (nalgebra)
  stats.rs        — Statistical distributions (statrs)
  optimize.rs     — 1D bounded optimization (argmin)
  linear.rs       — Continuous LP (good_lp + microlp)
  complex.rs      — Complex number ops (num-complex)

tests/
  rational_properties.rs       — proptest: algebraic laws
  calculus_properties.rs       — proptest: derivative rules
  [X]_properties.rs            — one file per module
  domain_error_codes.rs        — every ErrorCode variant must be tested here
  cli_contract.rs              — end-to-end CLI JSON round-trips
  protocol_properties.rs       — contract_version stability
```

---

## How to add a new command — the full checklist

Follow this order. Do not skip steps.

```
[ ] 1. src/X.rs
        - Define XRequest (serde Deserialize), XResponse (serde Serialize)
        - impl XRequest { pub fn evaluate(&self) -> XResponse }
        - All error variants return XResponse::Error { code: ErrorCode::_, reason: String }
        - Exact outputs: include { numerator, denominator, display } + no exactness field
        - Approx outputs: include exactness: "approximate_f64"
        - Add X_schema_json() function emitting a JSON Schema object with title
          "agent-calc calc1 X request"

[ ] 2. src/lib.rs
        - pub mod X;
        - pub use X::{XRequest, XResponse, x_schema_json};

[ ] 3. src/main.rs
        - Add Some("X") arm in main() calling x(&args[1..])
        - Add fn x(args: &[String]) -> ExitCode following the existing pattern
        - Add [domain] if domain == "X" arm in schema() calling x_schema_json()
        - Update print_usage() help text

[ ] 4. src/protocol.rs
        - Add "evaluate-X-operations" to Describe::capabilities
        - Add "X-safe: <one-line invariant>" to Describe::invariants

[ ] 5. tests/X_properties.rs
        - At minimum three property tests using proptest generators
        - Required properties: round-trip, identity, boundary
        - Generators must cover negative inputs, zero, large values
        - Do NOT use example-based tests as a substitute

[ ] 6. tests/domain_error_codes.rs
        - Add at least one assert for every XResponse::Error variant
        - Cover: invalid input, division by zero (if applicable), unsupported

[ ] 7. tests/cli_contract.rs
        - schema_X_emits_X_request_schema() — verify title field
        - X_golden_case() — one stdin→stdout round-trip asserting key fields
        - X_bad_json_exits_2() — malformed JSON returns exit code 2

[ ] 8. Run make full
        - All tests pass
        - No surviving mutants on the new module's core path
        - fmt-check clean
```

---

## Testing philosophy

**Property-based first.** Every module's behavior is specified as algebraic
properties over generated inputs, not a list of examples. An example test
confirms one case. A property test explores thousands of cases and can shrink
to a minimal counterexample.

**What a good property looks like for this codebase:**

```rust
// Good: tests an algebraic law with a generator
proptest! {
    fn addition_is_commutative(a in rational_strategy(), b in rational_strategy()) {
        prop_assert_eq!(eval(add(a, b)), eval(add(b, a)));
    }
}

// Bad: disguised example test — do not write these
#[test]
fn two_plus_three_is_five() {
    assert_eq!(eval(add(int(2), int(3))), 5);
}
```

**Required property categories per module:**

| Category | What to test |
|---|---|
| Identity | `x + 0 = x`, `x * 1 = x`, derivative of constant = 0 |
| Commutativity / symmetry | where the operation is symmetric |
| Associativity | where applicable |
| Round-trip | `eval(substitute(expr, bindings))` == direct eval |
| Boundary | zero denominators, empty inputs, max-size inputs |
| Error coverage | every `ErrorCode` variant reachable via a generator |

**Mutation testing as a pressure gauge.** After implementing a module, run
`make mutants`. A surviving mutant does not mean "the mutant is correct" — it
means one of:

1. The property generator is not exploring the relevant state space.
2. The assertion is too loose (asserting `!= 0` when it should assert `== 5`).
3. The behavior is genuinely unspecified (fix this by adding a property).

Do not add `#[mutants::skip]` to hide a surviving mutant. Investigate it.

---

## Exactness contract — reference table

| Module | Result type | Required fields | exactness field |
|---|---|---|---|
| eval, finance, interval | rational | `numerator`, `denominator`, `display` | absent |
| matrix, complex, stats, optimize, linear | f64 | value/data/re/im | `"approximate_f64"` required |
| polynomial roots (degree ≤ 2) | rational | per root | absent |
| units | f64 | value, unit | `"approximate_f64"` required |
| calculus | expr AST | unchanged | absent (symbolic) |
| simplify | expr AST | unchanged | absent (symbolic) |

When adding a new module: decide exactness at the design stage, not
implementation. Write the property test for the exactness contract before
writing the handler.

---

## Error codes — stable contract

`ErrorCode` variants are part of the public contract (`contract_version`).
Never rename or remove a variant. Never change what a code means.

Current stable codes:
- `DivisionByZero` — denominator is zero, interval contains zero in divisor
- `UnboundSymbol` — symbol without a binding at eval time
- `Overflow` — integer digit count exceeds limit
- `InputTooLarge` — expression depth/node count/binding count exceeds limit
- `InvalidInput` — bad parameter value (negative std_dev, singular matrix, etc.)
- `InvalidSymbol` — symbol name fails the identifier validity check
- `Unsupported` — operation is valid in principle but not implemented (e.g. nonlinear solve)
- `NoSolution` — affine system with no solution
- `InfiniteSolutions` — affine system with infinite solutions

Adding a new code: add the variant to `ErrorCode`, add a test in
`domain_error_codes.rs`, document in this file.

---

## Make targets — when to use what

| Target | When |
|---|---|
| `make test` | After any edit — fastest full check |
| `make check` | After structural changes — catches type errors without running tests |
| `make ci` | Before committing — fmt + check + clippy + test + package |
| `make full` | Before marking a feature done — ci + mutation testing |
| `make smoke` | After `make install` — verify the binary is wired up |
| `make mutants` | After implementing a new module or property test |
| `make clippy` | Periodically — catches lint warnings |
| `make audit` | Before releases — supply chain check |
| `make install-hooks` | Once per clone — wires up the git hooks |

## Git hooks — local quality gates

All enforcement is local. No CI. Install once per clone:

```bash
make install-hooks
```

This sets `git config core.hooksPath hooks` so git uses the `hooks/` directory
in the repo root — no file copying, and hooks stay in version control.

**`hooks/pre-commit`** — runs on every `git commit`:
- `cargo fmt --check`
- `cargo check --all-targets`
- `cargo clippy -- -D warnings`
- `cargo test --locked`
- `cargo package`
- 4 contract smoke checks against the debug binary

Takes ~30–60s. This is the full gate. Mutation is excluded because it is too
slow for every commit.

**`hooks/pre-push`** — runs when pushing to `main`:
- Runs `cargo mutants` scoped to the `src/` files changed since `main`
- On feature branches: skips automatically (logs a reminder instead)
- Force on any push: `AGENT_CALC_MUTANTS=1 git push`

To bypass in an emergency (with intent): `git commit --no-verify`

---

## Schema contracts — breaking vs non-breaking

**Breaking changes** (require contract_version bump):
- Removing a field from any response type
- Renaming a field
- Changing a field's type
- Removing an `intent` variant
- Changing an `ErrorCode` variant name or meaning

**Non-breaking** (safe to ship without version bump):
- Adding a new optional response field
- Adding a new `intent` variant
- Adding a new `ErrorCode` variant
- Adding a new command entirely

When making a breaking change, update `contract_version` in `protocol.rs` and
update `cli_contract.rs` to assert the new version.

---

## What Claude should do in this project

- Run `make ci` before reporting a task complete. If it fails, fix it.
- Write the property test file (`tests/X_properties.rs`) before or alongside
  the implementation — not after.
- When a mutant survives, read the mutant output and strengthen the property.
- When adding a field to a response, check the exactness contract table above.
- Check `domain_error_codes.rs` after adding any new error variant.
- Prefer editing existing files to creating new ones. Prefer adding a property
  to an existing `_properties.rs` file over creating a parallel file.

## What Claude must not do

- Do not skip `fmt-check`. Run `make fmt` to fix formatting before committing.
- Do not mark a command complete without a `tests/X_properties.rs` entry.
- Do not add `#[mutants::skip]` to pass mutation testing.
- Do not use `--no-verify` on git commands.
- Do not weaken a property assertion to make it pass against a mutant.
- Do not use `f64` for a result that the protocol promises is exact rational.
- Do not add prose error messages to stdout — errors go to stderr only.
