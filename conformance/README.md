# Conformance

Three transports, one kernel: the CLI, the WebAssembly build, and `calcd` all
call `execute_tool_json`. That is the design. This is the evidence.

    cargo build --release --bin agent-calc
    cargo build --release --features server --bin calcd
    ./target/release/calcd &
    node conformance/run.mjs

The corpus is not invented — it is every request shape bento actually issues,
captured from its four worlds, because the shapes a real caller sends are the
ones worth guaranteeing.

## What it found on its first run

3 of 36 cases diverge between the CLI binary and the calcd binary, in the last
one or two bits of an f64:

    iqr        1.6999999999999997  vs  1.6999999999999995
    iqr        1.9250000000000003  vs  1.9250000000000005
    lp var    28780.235014399746   vs 28780.235014399743

WASM matched the CLI on all 36. Only the two native binaries disagree.

Ruled out by measurement, in this order: input encoding (the strings reaching
the kernel are byte-identical, verified by echoing both); Cargo feature
unification (no maths crate gains or loses a feature under `--features server`);
the FPU control word (MXCSR is 0x1f80 on both the main thread and the workers —
no flush-to-zero, no denormals-are-zero, round-to-nearest); and thread identity
(the divergence reproduces on a single thread).

What is left is codegen: the same Rust source, compiled into two binaries with
different dependency graphs, emits float sequences that round differently in the
last bit. It is stable — the same binary always gives the same answer — but it
is not portable between binaries.

## The policy that follows from it

Two tiers, because these are not the same kind of problem.

**MUST** — anything that changes a decision or a displayed figure. Feasible vs
infeasible, a survivor count, a dollar. A failure blocks.

**NOTE** — the last bit or two of an f64, below anything a caller can act on.
Recorded, not blocking. Blocking on it would mean a permanently red build that
nobody can fix and everybody learns to ignore, which is worse than one that
reports honestly.

Measured in the product rather than assumed: routing a 121-cell survival map
through calcd instead of the browser moved 13 cells in the fifteenth significant
figure and changed no feasibility, no survivor count, and no edge.

## Why this matters less than it first appeared

The first reading of this was that it threatened the seal: a sealed pack is a
signature over a hash of the figures, and a hash has no bit that matters less
than the others.

That reading was wrong, and checking beat assuming. Verification re-hashes the
claims carried inside the pack; it never re-solves the model. The figures travel
with the signature. So two transports disagreeing in the fifteenth figure cannot
make a genuine pack read as tampered — there is nothing to disagree with at
verification time.

What remains is worth stating plainly rather than dressing up: the arithmetic is
stable within a binary and not quite portable between binaries, at a magnitude
no caller can act on. Conformance runs so that if it ever becomes a magnitude
somebody can act on, the build says so.

Non-goals for this harness: it does not check performance, and it does not check
that the answers are *correct* — only that they are the same everywhere. The
domain tests do the former.
