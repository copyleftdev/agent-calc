# agent-calc

`agent-calc` is an AI-native exact calculator and contract-first Rust CLI for
deterministic symbolic math, exact rational arithmetic, typed JSON schemas,
property-based testing, and mutation-tested computation workflows.

Use it when an AI agent needs a calculator that returns machine-readable
contracts instead of prose guesses: exact evaluation, symbolic simplification,
traceable proof steps, assumptions, affine solving, calculus, inequalities,
polynomials, intervals, finance, units, matrices, statistics, optimization,
linear programming, and complex numbers.

## Search Hooks

AI calculator, exact calculator, Rust calculator CLI, deterministic math engine,
typed JSON math API, contract-first CLI, AI tool contract, symbolic math Rust,
exact rational arithmetic, property-based testing, mutation testing,
machine-readable math, schema-driven CLI, agent tool executable, calculator for
LLMs, proof trace calculator, finance calculator CLI, matrix calculator Rust,
statistics calculator CLI, units conversion CLI, linear programming CLI,
complex-number calculator, polynomial solver, interval arithmetic, assumptions
engine, affine equation solver.

## About

`agent-calc` gives AI systems a bounded computation tool with stable command
contracts. It avoids hand-wavy answers by accepting typed JSON, validating
inputs, enforcing runtime limits, and returning typed JSON with exact results,
stable error codes, and optional trace steps.

The architecture follows a contract-first CLI pattern:

- `agent-calc describe` emits the executable contract.
- `agent-calc schema` emits the request schema.
- `agent-calc eval` evaluates typed JSON and returns typed JSON.
- `agent-calc simplify` applies bounded symbolic identities and returns typed JSON.
- `agent-calc trace` evaluates or simplifies expressions with deterministic trace steps.
- `agent-calc substitute` applies exact bindings to symbolic expressions.
- `agent-calc assumptions` validates symbolic assumption contexts.
- `agent-calc solve` solves exact affine symbolic equations.
- `agent-calc calculus` differentiates symbolic expressions.
- `agent-calc inequality` solves exact affine inequalities.
- `agent-calc polynomial` operates on exact rational polynomials.
- `agent-calc interval` analyzes exact rational closed intervals.
- `agent-calc finance` evaluates exact time-value finance requests.

Core expression requests return stable machine-readable error codes alongside
human-readable reasons. The shared expression protocol also enforces runtime
limits for decimal precision, expression depth, expression node count, integer
digit count, symbol length, exponent size, and substitution binding count.

The `eval` slice supports exact rational arithmetic, including integer powers,
with explicit error states. The `simplify` slice supports bounded symbolic
identities such as `x + 0 -> x` and `x * 1 -> x`. The `trace` slice wraps
evaluation and simplification with ordered, machine-readable proof steps so an
AI caller can inspect which exact rule produced each intermediate result.

The `substitute` slice requires every symbol to be bound to an exact evaluable
expression, then applies the current simplifier to the substituted expression.

The `assumptions` slice validates typed symbolic domain facts and comparisons,
derives exact rational bounds and exclusions, and answers simple entailment
queries such as whether `x >= 2` implies `x > 0`.

The `solve` slice solves single-variable affine equations over the existing
expression AST and returns typed no-solution, infinite-solution, or unsupported
nonlinear failures.

The `calculus` slice currently supports exact symbolic derivatives over the
expression AST, including sum, difference, product, quotient, negation, and
integer-power rules.

The `inequality` slice solves single-variable affine inequalities and returns
typed exact solution sets: empty, all reals, point, not-point, or bounded
intervals with inclusive boundary metadata.

The `polynomial` slice represents coefficients in ascending power order and
supports exact normalization, addition, multiplication, evaluation, and
derivatives over rational coefficients. It also solves degree 0, 1, and 2
equations when all returned roots are exact rationals.

The `interval` slice supports exact closed-interval arithmetic and polynomial
range analysis over closed rational domains, including exact critical points
found through the polynomial derivative.

The `finance` slice supports exact rational future value, present value,
discount factors, and net present value over typed expression cash flows.

Unit conversion and dimension-checked quantity arithmetic are exposed through
`agent-calc units` and are backed by `uom`.

Matrix operations are exposed through `agent-calc matrix` and are backed by
`nalgebra`.

Statistics and probability operations are exposed through `agent-calc stats`
and are backed by `statrs`.

Bounded one-dimensional optimization is exposed through `agent-calc optimize`
and is backed by `argmin`.

Continuous linear programming is exposed through `agent-calc linear` and is
modeled with `good_lp` using the pure-Rust `microlp` solver backend.

Complex-number operations are exposed through `agent-calc complex` and are
backed by `num-complex`.

## Example

```bash
printf '%s\n' '{
  "expr": {
    "kind": "pow",
    "base": { "kind": "rational", "numerator": "2", "denominator": "3" },
    "exponent": -2
  }
}' | cargo run -- eval
```

```bash
printf '%s\n' '{
  "expr": {
    "kind": "mul",
    "left": { "kind": "symbol", "name": "x" },
    "right": { "kind": "integer", "value": "1" }
  }
}' | cargo run -- simplify
```

```bash
printf '%s\n' '{
  "intent": "eval",
  "expr": {
    "kind": "add",
    "left": { "kind": "integer", "value": "2" },
    "right": { "kind": "integer", "value": "3" }
  }
}' | cargo run -- trace
```

```bash
printf '%s\n' '{
  "expr": {
    "kind": "mul",
    "left": { "kind": "symbol", "name": "x" },
    "right": { "kind": "integer", "value": "1" }
  },
  "bindings": {
    "x": { "kind": "rational", "numerator": "2", "denominator": "3" }
  }
}' | cargo run -- substitute
```

```bash
printf '%s\n' '{
  "intent": "bounds",
  "symbol": "x",
  "assumptions": [
    { "kind": "domain", "symbol": "x", "domain": "positive" },
    {
      "kind": "compare",
      "symbol": "x",
      "op": "lte",
      "value": { "kind": "integer", "value": "5" }
    }
  ]
}' | cargo run -- assumptions
```

```bash
printf '%s\n' '{
  "intent": "solve",
  "variable": "x",
  "equation": {
    "left": {
      "kind": "add",
      "left": {
        "kind": "mul",
        "left": { "kind": "integer", "value": "2" },
        "right": { "kind": "symbol", "name": "x" }
      },
      "right": { "kind": "integer", "value": "3" }
    },
    "right": { "kind": "integer", "value": "7" }
  }
}' | cargo run -- solve
```

```bash
printf '%s\n' '{
  "intent": "derivative",
  "variable": "x",
  "expr": {
    "kind": "add",
    "left": {
      "kind": "pow",
      "base": { "kind": "symbol", "name": "x" },
      "exponent": 3
    },
    "right": {
      "kind": "mul",
      "left": { "kind": "integer", "value": "2" },
      "right": { "kind": "symbol", "name": "x" }
    }
  }
}' | cargo run -- calculus
```

```bash
printf '%s\n' '{
  "intent": "solve",
  "variable": "x",
  "inequality": {
    "left": {
      "kind": "add",
      "left": {
        "kind": "mul",
        "left": { "kind": "integer", "value": "2" },
        "right": { "kind": "symbol", "name": "x" }
      },
      "right": { "kind": "integer", "value": "3" }
    },
    "relation": "lt",
    "right": { "kind": "integer", "value": "7" }
  }
}' | cargo run -- inequality
```

```bash
printf '%s\n' '{
  "intent": "mul",
  "left": {
    "variable": "x",
    "coefficients": [
      { "kind": "integer", "value": "1" },
      { "kind": "integer", "value": "1" }
    ]
  },
  "right": {
    "variable": "x",
    "coefficients": [
      { "kind": "integer", "value": "1" },
      { "kind": "integer", "value": "-1" }
    ]
  }
}' | cargo run -- polynomial
```

```bash
printf '%s\n' '{
  "intent": "polynomial_range",
  "polynomial": {
    "variable": "x",
    "coefficients": [
      { "kind": "integer", "value": "0" },
      { "kind": "integer", "value": "-2" },
      { "kind": "integer", "value": "1" }
    ]
  },
  "domain": {
    "lower": { "kind": "integer", "value": "0" },
    "upper": { "kind": "integer", "value": "3" }
  }
}' | cargo run -- interval
```

```bash
printf '%s\n' '{
  "intent": "net_present_value",
  "rate": { "kind": "rational", "numerator": "1", "denominator": "10" },
  "cash_flows": [
    { "kind": "integer", "value": "-100" },
    { "kind": "integer", "value": "55" },
    { "kind": "rational", "numerator": "121", "denominator": "2" }
  ]
}' | cargo run -- finance
```

```bash
printf '%s\n' '{
  "intent": "solve",
  "polynomial": {
    "variable": "x",
    "coefficients": [
      { "kind": "integer", "value": "6" },
      { "kind": "integer", "value": "-5" },
      { "kind": "integer", "value": "1" }
    ]
  }
}' | cargo run -- polynomial
```

```bash
printf '%s\n' '{
  "intent": "convert",
  "quantity": { "dimension": "length", "value": 1.5, "unit": "km" },
  "to": "m"
}' | cargo run -- units
```

```bash
printf '%s\n' '{
  "intent": "mul",
  "left": { "rows": 2, "cols": 2, "data": [1, 2, 3, 4] },
  "right": { "rows": 2, "cols": 2, "data": [5, 6, 7, 8] }
}' | cargo run -- matrix
```

```bash
printf '%s\n' '{
  "intent": "normal_cdf",
  "mean": 0,
  "std_dev": 1,
  "x": 0
}' | cargo run -- stats
```

```bash
printf '%s\n' '{
  "intent": "minimize_1d",
  "objective": { "kind": "quadratic", "a": 1, "b": -6, "c": 9 },
  "lower": -10,
  "upper": 10
}' | cargo run -- optimize
```

```bash
printf '%s\n' '{
  "intent": "solve_lp",
  "direction": "maximize",
  "objective": [10, 11],
  "variables": [{ "lower": 0 }, { "lower": 0 }],
  "constraints": [
    { "coefficients": [1, 2], "relation": "le", "rhs": 5 },
    { "coefficients": [1, 1], "relation": "le", "rhs": 3 }
  ]
}' | cargo run -- linear
```

```bash
printf '%s\n' '{
  "intent": "mul",
  "left": { "re": 1, "im": 2 },
  "right": { "re": 3, "im": 4 }
}' | cargo run -- complex
```

## Testing

The baseline test posture is property-first:

```bash
cargo test
```

Mutation testing should be used as pressure against weak properties:

```bash
cargo mutants
```

The mutation goal is not just line coverage. A surviving mutant means either the
property is too weak, the generator is not exploring the relevant state, or the
behavior is not specified tightly enough.
