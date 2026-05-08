---
name: New command
about: Propose a new agent-calc command or intent
title: "feat(X): "
labels: enhancement
---

## What the command does

<!-- One paragraph: what problem it solves, what archetype of caller needs it. -->

## Proposed JSON contract

<!-- Show the request and response shape. Be specific. -->

**Request:**
```json
{
  "intent": "...",
  "...": "..."
}
```

**Response (success):**
```json
{
  "status": "...",
  "contract_version": "calc1/0.1.0",
  "...": "..."
}
```

**Response (typed failure):**
```json
{
  "status": "error",
  "code": "...",
  "reason": "..."
}
```

## Exactness

- [ ] Exact rational output (no `exactness` field)
- [ ] Approximate f64 output (`"exactness": "approximate_f64"` required)
- [ ] Mixed (document which fields are exact and which are approximate)

## Invariant statement

<!-- Complete the sentence: "X-safe: ..." as it would appear in Describe::invariants -->

```
X-safe: ...
```

## Error cases

<!-- List every ErrorCode this command can return and what triggers it. -->

| Code | Trigger |
|---|---|
| `InvalidInput` | |
| `DivisionByZero` | |
| `Unsupported` | |

## Properties required

<!-- List the algebraic properties that tests/X_properties.rs must cover. -->

- [ ] ...identity...
- [ ] ...round-trip...
- [ ] ...boundary (zero / negative / max-size)...

## Crate dependencies

<!-- List any crates this requires. Note if they are already in Cargo.toml. -->

## Use cases by role

<!-- From the roleplay sessions or real-world archetypes. -->
