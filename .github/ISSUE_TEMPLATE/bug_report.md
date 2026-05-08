---
name: Bug report
about: Report incorrect output, wrong error code, schema mismatch, or contract violation
title: "fix(X): "
labels: bug
---

## Command and intent

```
agent-calc <command>
```

## Input JSON

```json

```

## Actual output

```json

```

## Expected output

<!-- What the contract / invariants say should happen. -->

```json

```

## Contract violation

- [ ] Wrong `status` value
- [ ] Wrong `code` in error response
- [ ] `exactness` field missing on approximate result
- [ ] `exactness` field present on exact rational result
- [ ] `contract_version` field missing
- [ ] Schema mismatch (field name or type differs from `agent-calc schema X`)
- [ ] Exit code wrong (expected 0 for typed error, 2 for bad JSON)
- [ ] Panic / non-JSON output on stdout
- [ ] Other invariant broken (describe below)

## Reproduction

```bash
printf '%s' '<input json>' | agent-calc <command>
```

## Version

```
agent-calc --version
```
