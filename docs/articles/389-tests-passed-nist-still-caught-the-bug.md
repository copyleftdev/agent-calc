---
title: 389 Tests Passed. NIST Still Caught the Bug.
published: false
description: What stress-testing a calculator for AI agents taught me about independent reference data, mutation gates, and sources of truth whose authority can be revoked by evidence.
tags: ai, testing, tooling, rust
cover_image: https://raw.githubusercontent.com/copyleftdev/agent-calc/main/assets/article/oddly-exact-dev-cover-reference-beam-1000x420-v2.png
---

I gave an AI agent a calculator because I wanted one hard, inspectable point
inside a probabilistic workflow.

The model could interpret the request and explain the result. The calculator
would perform the computation. It seemed like a clean division of labor.

Then I changed one multiplication sign into addition.

The calculator still passed 389 of the 390 tests in its Rust library harness.
The sole failure compared its answer with NIST's certified results for the
Longley regression dataset.

That bothered me more than a completely broken build would have. I had treated
deterministic computation as safer than asking a language model to improvise
arithmetic. But deterministic does not mean trustworthy. A program can return
the same wrong answer forever.

“Source of truth” suddenly felt too comfortable. Before an AI agent delegates
authority to a tool, that authority should be challenged—and remain revocable
by evidence.

The calculator is only the specimen. The larger idea is a way to place
inspectable, replayable instruments inside probabilistic systems.

## The useful boundary is generation versus execution

The interesting distinction is not model weights versus a “real CPU.” Model
inference also runs on processors, and language models can learn genuine
arithmetic procedures. The useful boundary is between **generating an answer**
and **executing a defined operation under a tested contract**.

Research on [Program-Aided Language Models
(PAL)](https://proceedings.mlr.press/v202/gao23f.html) makes a related split:
the language model reads and decomposes a natural-language problem, while a
runtime such as a Python interpreter executes the generated program. The model
contributes flexible interpretation; the runtime contributes executable
semantics.

That is the division I want in an agent:

- At the **semantic edge**, the model interprets the request, chooses a
  procedure, identifies relevant quantities, and explains the result.
- At the **computational edge**, a narrow tool validates inputs, applies
  specified operations, enforces limits, and returns structured output.

This does not make the whole agent deterministic, and it does not make the
model unnecessary. The agent can still choose the wrong tool, supply the wrong
arguments, misunderstand units, or misread the result.

The promise is smaller: one claim becomes inspectable, replayable, and
independently testable.

> The opposite of probabilistic is not trustworthy. It is repeatable.
>
> A CPU can be precisely wrong.

## Why NIST became my external witness

Calibration cannot be entirely self-referential. The implementation should not
be the sole author of its own expected answers.

That is why I chose NIST—not because government authority turns a result into
mathematical truth, but because NIST has a long institutional practice of
building shared, independently evaluated references.

[Congress established the National Bureau of Standards in
1901](https://www.nist.gov/pml/owm/about-owm/owm-background-and-history) to
strengthen the United States' measurement infrastructure. The [Standard
Reference Data Act of
1968](https://www.nist.gov/srd/srd-definition) authorized a federal program to
collect, critically evaluate, publish, and distribute standardized scientific
and technical reference data. NBS [became NIST in
1988](https://www.nist.gov/timeline). In 1999, that lineage reached statistical
software through the [Statistical Reference Datasets
project](https://www.itl.nist.gov/div898/strd/), usually shortened to StRD.

StRD pairs datasets with certified expected values for specific statistical
procedures. Its collection includes generated and real-world cases of varying
difficulty. For linear procedures, NIST carried [500 digits through its
calculations](https://www.nist.gov/itl/sed/products-services/statistical-reference-data-sets-strd)
so ordinary floating-point representation error would not become the
benchmark.

Longley is a small but numerically challenging linear-regression dataset in
that collection. Its certified results gave my tests something the
implementation could not manufacture for itself: an expected answer produced
outside the code under test.

That qualification matters. StRD does **not** certify Oddly Exact, prove the
statistics engine correct, or make the software traceable to NIST. NIST
explicitly says the datasets are an aid for evaluating software and that [no
mechanism establishes software
traceability](https://www.itl.nist.gov/div898/strd/general/faq.html).

The reference data was not an oracle for the entire program. It was an
independent witness for the calculations it covered.

## Test the tests

Example-based tests ask whether familiar inputs still produce familiar
outputs. Mutation testing asks a more uncomfortable question:

**If I introduce a small, plausible defect, does the suite notice?**

I used `cargo-mutants` to replace multiplication with addition in the
multiple-regression standard-error calculation:

```diff
- residual_std_dev * sum_sq.sqrt()
+ residual_std_dev + sum_sq.sqrt()
```

The mutation preserved valid Rust, valid types, and a plausible-looking numeric
result. In the library harness, 389 of 390 tests still passed. The assertion
against NIST's Longley values was the sole failure.

The [public mutation
record](https://gist.github.com/copyleftdev/277e48079658f6f89deb2254701ebcf7)
preserves the exact command, environment, result, and the limit of what that
experiment establishes:

{% embed https://gist.github.com/copyleftdev/277e48079658f6f89deb2254701ebcf7 %}

I cannot claim Longley was the only test in the entire repository capable of
catching the mutation; the run stopped after the failed library harness. I can
claim something narrower and more useful: hundreds of tests tolerated a
semantically broken formula, while an assertion anchored to independently
produced values rejected it.

A green test count is evidence only to the extent that those tests would turn
red when the implementation meaningfully changes. Mutation testing measures
that sensitivity instead of admiring the count.

## Challenge the contract around the answer

The formula mutation challenged numerical meaning. My next audit challenged
the promises around the calculation: what the tool accepts, what it refuses,
and how much work it will perform.

The first crack was a contradiction between the advertised contract and the
executable one. The generated JSON Schema forbade additional properties, but
the Rust deserializer silently accepted an unknown field at the request root
and another inside an expression node.

The answer was still numerically correct. That did not make the behavior
harmless. A caller validating against the schema saw a stricter instrument than
a caller speaking directly to the binary. A misspelled or misunderstood field
could disappear without warning.

For an agent-facing tool, silently interpreting a different contract is itself
a correctness defect.

The second crack was a resource boundary. The expression engine limited things
such as integer size, expression depth, and precision. The optimization
interface, however, accepted grid resolution and iteration counts without upper
ceilings.

I requested a grid search with one billion sample points. The pre-repair binary
produced no JSON before an external one-second watchdog terminated it. The tool
had input validation, but it had not earned the bounded-work guarantee I
thought I had built.

Those failures became permanent regression requests. The deserializer now
rejects unknown fields at both levels. The optimizer now publishes and enforces
ceilings of 100,000 iterations and 1,000,000 grid points. The same billion-point
request returns a typed `resource_limit` response before entering the search.

The [contract challenge
Gist](https://gist.github.com/copyleftdev/28ed6931c6174099b21db54846f72b65)
is a runnable replay of all three repaired cases:

{% embed https://gist.github.com/copyleftdev/28ed6931c6174099b21db54846f72b65 %}

That script is more valuable than a screenshot of a green run. It lets another
observer cross-examine the boundary directly.

## Repair is the next claim to attack

A new guard is only another claim until the tests prove they care about it.

After repairing the optimizer limits, I selected every mutation
`cargo-mutants` generated for the two new validators. Eleven mutations tried to
remove the checks, replace them with unconditional success, or alter their
boundary comparisons. The tests caught all eleven.

The [focused optimizer mutation
record](https://gist.github.com/copyleftdev/9b9afe1b1173217d92a4cb0ede2278d0)
captures the selection and outcome:

{% embed https://gist.github.com/copyleftdev/9b9afe1b1173217d92a4cb0ede2278d0 %}

That completed the loop:

**claim → attack → fail → repair → mutate → replay**

If I published only the final green suite, you would see confidence. By
preserving the changed operator, the request that hung, the contract
contradiction, the repair, and the mutations that tried to undo it, I can show
a reason for confidence.

The scar is part of the calibration record.

## A five-layer challenge stack for agent tools

Oddly Exact is one calculator, but the method travels. Before giving a narrow
tool authority inside an agent workflow, I now ask five questions:

1. **Is the contract explicit and executable?** The schema, parser, runtime,
   limits, and failure modes must agree. Documentation the executable does not
   enforce is only a suggestion.

2. **Is there an independent reference?** In this statistical case, NIST StRD
   moved selected expected answers outside my implementation. Another domain
   might use a standards specification, a reference implementation, a verified
   corpus, or a separately derived test oracle.

3. **Which properties should survive across many inputs?** Property-based tests
   generate cases and check laws rather than memorizing individual examples.
   One Oddly Exact property checks that translating every value in a sample by
   a large exact offset leaves sample variance unchanged within a tight
   floating-point tolerance.

4. **Can the tests detect plausible corruption?** Mutation testing changes
   operators, comparisons, return values, and guards. A surviving mutation does
   not prove the implementation is wrong; it exposes a behavior change the
   suite cannot currently distinguish.

5. **Are failures bounded, typed, and replayable?** An agent must distinguish a
   rejected computation from an unavailable tool. Division by zero, resource
   exhaustion, malformed JSON, and a crashed process should not collapse into
   the same ambiguous failure.

These layers do different jobs. External references challenge specific
answers. Properties challenge behavior across an input space. Mutations
challenge the tests themselves. Contract and resource attacks challenge the
boundary around all of it.

None provides universal correctness. Together they create a narrower and more
useful result: evidence that a particular tool deserves provisional authority
for validated requests inside a declared contract.

## A source under challenge

For a validated request inside that contract, the tool's structured result can
serve as the agent's operational source of truth.

That is a runtime role, not a claim of infallibility. If the agent encounters a
disagreement, it can inspect the arguments, surface the conflict, or consult
another independently trusted tool. It should not quietly replace a structured
result with fresh prose arithmetic.

The trust relationship runs in opposite directions at different times:

- While building the tool, the developer should distrust it aggressively.
- Inside a contract the tool has earned, the agent should respect its answer or
  its typed refusal.

New evidence can always return the tool to the first phase. That is a **source
under challenge**: trusted in operation, open to appeal, and always one
counterexample away from revision.

The beautiful thing is not that source code is truth.

The beautiful thing is that source code can be cross-examined.

I did not end this experiment with a calculator an AI agent can trust forever.
I ended it with something more useful: [a calculator whose authority can be
revoked by evidence](https://github.com/copyleftdev/agent-calc).

---

*AI tools assisted with editorial research and revision. I verified the
technical claims and stand behind the final text.*
