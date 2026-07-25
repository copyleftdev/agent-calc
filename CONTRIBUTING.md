# Contributing to Oddly Exact

## Development philosophy

Oddly Exact is the product; `agent-calc` is its stable contract-first CLI.
Every design decision flows from three commitments:

1. **Exact-first** — represent results as exact rationals when possible.
2. **Typed failures** — every error path returns a stable `ErrorCode`.
3. **Bounded inputs** — enforce runtime limits so the tool is safe to call
   from untrusted agents.

Before writing any code, decide: what are the algebraic properties this module
must satisfy? Write those properties as proptest tests first. The implementation
follows from the properties.

---

## Prerequisites

```bash
rustup toolchain install stable
rustup component add rustfmt clippy
cargo install cargo-mutants  # for mutation testing
```

---

## Setup (once per clone)

```bash
make install-hooks
```

This runs `git config core.hooksPath hooks` so git uses the `hooks/` directory.
The hooks live in version control — no separate install script to maintain.

## Daily workflow

```bash
make test         # after any edit — fast feedback
make ci           # same as pre-commit, run manually any time
make full         # before claiming a feature is done (ci + mutants)
```

`make ci` = `fmt-check + check + clippy + test + package`
`make full` = `ci + mutants`

## What the hooks enforce

**`hooks/pre-commit`** fires on every `git commit` and runs the full gate:
fmt-check → check → clippy → test → package → 4 contract smoke checks.

**`hooks/pre-push`** fires when pushing to `main` and runs mutation testing
scoped to the `src/` files changed since `main`. Feature branches skip it.
Force it on any branch: `AGENT_CALC_MUTANTS=1 git push`.

Emergency bypass (use sparingly): `git commit --no-verify`

---

## Adding a new command

Read the full checklist in `CLAUDE.md` before starting. The short version:

1. `src/X.rs` — types + `evaluate()` + `x_schema_json()`
2. Wire into `src/lib.rs`, `src/main.rs`, `src/protocol.rs`
3. `tests/X_properties.rs` — three or more proptest properties
4. `tests/domain_error_codes.rs` — every new `ErrorCode` variant
5. `tests/cli_contract.rs` — schema, round-trip, bad-JSON golden tests
6. `make full`

---

## Testing requirements

### Property-based tests (required)

Every module needs a `tests/X_properties.rs` file with `proptest!` blocks.

Each property must:
- Use a generator that covers zero, negative, boundary, and large values
- Assert an algebraic law (commutativity, identity, round-trip, etc.)
- Be tight enough that a relevant mutation kills it

**A property that passes against a mutant is not finished.**

### Mutation testing standard

Run `make mutants` before marking a feature complete. Surviving mutants are
bugs in the test suite, not in the implementation.

- Do not add `#[mutants::skip]` to silence a surviving mutant.
- Do not weaken an assertion to avoid a failure against a mutant.
- Do tighten the generator or assertion until the mutant is killed.

### Error code coverage

`tests/domain_error_codes.rs` must have at least one test for every
`ErrorCode` variant reachable from each command. If you add a new variant,
add a test before the PR is submitted.

---

## Exactness contract

| Result type | `exactness` field | Required response fields |
|---|---|---|
| Exact rational | absent | `numerator`, `denominator`, `display` |
| Approximate f64 | `"approximate_f64"` | `value` or `data` |

Omitting `exactness` on an f64 result is a contract violation. Callers rely
on its presence to decide whether to trust the result for exact downstream
arithmetic.

---

## Schema stability

The `contract_version` field (`calc1/0.1.0`) governs breaking changes.

**Breaking** (bump version):
- Removing or renaming a response field
- Changing a field's type
- Removing an `intent` variant
- Changing an `ErrorCode` meaning

**Non-breaking** (no bump needed):
- Adding a new optional response field
- Adding a new `intent` or command
- Adding a new `ErrorCode` variant

---

## Code style

- `cargo fmt` before every commit.
- `cargo clippy -- -D warnings` must be clean.
- No `unwrap()` or `expect()` outside of test code.
- No `println!` in library or handler code — only `eprintln!` for errors.
- Errors go to stderr. JSON output goes to stdout. Always.

---

## Quality gates

All gates are local git hooks — no remote CI.

| Hook | Fires | Runs |
|---|---|---|
| `pre-commit` | every `git commit` | fmt + check + clippy + test + package + smoke |
| `pre-push` | push to `main` | `cargo mutants` on changed src/ files |

Run `make audit` manually before any release to check supply-chain health.

---

## Commit message style

```
<type>(<scope>): <short description>

<optional body>
```

Types: `feat`, `fix`, `test`, `refactor`, `docs`, `chore`
Scope: the command or module name (`eval`, `finance`, `polynomial`, etc.)

Examples:
```
feat(calculus): add definite integral with exact rational bounds
fix(matrix): standardize solve field type to match MatrixInput
test(stats): strengthen describe_sample properties to kill median mutant
```
