## What this PR does

<!-- One paragraph. What changed and why. -->

## Checklist

### For every PR
- [ ] `make ci` passes locally (fmt-check + check + clippy + test + package)
- [ ] No new `unwrap()` / `expect()` calls outside of test code
- [ ] Errors returned as `XResponse::Error { code: ErrorCode::_, reason }` — not panics
- [ ] Prose goes to stderr only; stdout carries JSON only

### For new commands
- [ ] `src/X.rs` — request/response types + `evaluate()` + `X_schema_json()`
- [ ] `src/lib.rs` — module declared and types re-exported
- [ ] `src/main.rs` — dispatch arm + schema arm + help text updated
- [ ] `src/protocol.rs` — capability and invariant added to `Describe`
- [ ] `tests/X_properties.rs` — at minimum three proptest properties
- [ ] `tests/domain_error_codes.rs` — every new `ErrorCode` variant covered
- [ ] `tests/cli_contract.rs` — schema golden test + round-trip test + bad-JSON test

### For changes to existing commands
- [ ] Existing properties still pass without modification
- [ ] If a response field was added: field is optional or `contract_version` was bumped
- [ ] If a response field was removed or renamed: `contract_version` was bumped
- [ ] `tests/domain_error_codes.rs` updated for any new error variants

### For exactness-affecting changes
- [ ] Exact rational results: no `exactness` field present
- [ ] Approximate results: `"exactness": "approximate_f64"` present
- [ ] If a previously-approximate result is now exact (or vice versa): `contract_version` bumped

### Mutation testing (for new modules or significantly changed logic)
- [ ] `make mutants` run; surviving mutants investigated and properties strengthened
- [ ] No `#[mutants::skip]` added without a comment explaining why it is unreachable

## Contract version impact

- [ ] No breaking change — contract_version unchanged
- [ ] Breaking change — `contract_version` bumped and migration note added below

<!-- If breaking: describe what callers must update. -->
