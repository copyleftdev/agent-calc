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

## Why this matters more than the size of the number

A fifteenth decimal place is invisible in anything bento displays; every figure
on screen is rounded to dollars or a tenth of a percent. The exposure is the
seal. A sealed pack is a signature over a hash of the figures, and a hash does
not have a last bit that matters less than the others. The same model sealed
after a server solve and verified after a browser solve would produce two
different hashes and read as tampering.

So the rule this implies: a sealed artifact must be computed by one transport
end to end, and conformance has to gate deployment rather than run occasionally.

Non-goals for this harness: it does not check performance, and it does not check
that the answers are *correct* — only that they are the same everywhere. The
domain tests do the former.
