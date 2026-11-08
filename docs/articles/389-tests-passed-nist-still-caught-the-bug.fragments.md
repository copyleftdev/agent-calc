# Review fragments: green counts, sensitivity, and independent witnesses

“A green test count is evidence only to the extent that those tests would turn red” survives the challenge. The response sharpens the mechanism: a test catches a mutant only when the path is reached, the input makes the changed behavior diverge, the oracle is independent enough to disagree, and the assertion tolerance exposes the divergence.

---

The offset experiment demonstrates catastrophic cancellation in a naive one-pass variance formula. That is a useful adjacent example, but it is not a reproduction of the article’s mutation. The article changed a multiple-regression standard-error operation from multiplication to addition. Evidence about one mutant’s sensitivity profile does not establish why another mutant survived.

---

The response uses population variance, `(sum_sq - sum^2/n)/n`. The repository property exercises sample variance, whose denominator is `n - 1`. Translation invariance applies to both, but quoted outputs are not directly comparable unless the estimator, dataset, summation order, and tolerance are fixed.

---

The reported `8.4e-2` relative error does not match the accompanying values `9.43` and `9.60`. Their relative difference is about `1.80e-2`; treating them as standard deviations and comparing the corresponding variances gives about `3.64e-2`. The missing dataset or a clarified error definition is needed to reproduce the claim.

---

“Independence AND sensitivity” is right but incomplete as a test model. Reachability comes first. Most of the 389 green library tests do not execute the multiple-regression standard-error path at all. On that path, the broad local test checks only positivity, finiteness, and a loose upper bound. The NIST test compares all seven standard errors against independently certified values.

---

Longley’s ill-conditioning can amplify some numerical defects. It is not yet shown to be necessary for catching the article’s `* → +` mutation. Addition in place of multiplication changes the formula dimensionally and should be detectable on ordinary well-conditioned data when a sufficiently precise independent or frozen expected value is asserted.

---

A checked-in golden value does not have zero sensitivity by construction. If it was captured before a mutation and is not regenerated, it stays fixed and can catch the mutation. The zero-sensitivity case is a live self-referential oracle—or a snapshot automatically regenerated from the mutated implementation during the test—not a golden assertion in the usual sense.

---

“NIST’s value was the one nobody in the codebase could edit” is memorable but technically wrong. Copied constants can be edited. Their strength is independent provenance: the expected values were derived outside the implementation and can be audited against the source.

---

Ten thousand similar green inputs may add no discriminating power against one specific mutant under one oracle and tolerance. The scoped phrase matters. Those inputs might discriminate a different mutation, exercise different branches, or establish a distributional guarantee.

---

File and output hashes show that unseen bytes were stable across runs. They do not make the experiment independently reviewable without the bytes, dataset, exact assertion, tolerance, runtime version, and invocation.

---

The repository reproduces the article’s exact contrast under the `residual_std_dev * sqrt(sum_sq) → residual_std_dev + sqrt(sum_sq)` mutation. `multiple_regression_std_errors_are_positive` still passes. `multiple_regression_nist_longley_passes_certified_values` fails on the first standard error with relative error about `9.96e-1`: mutated `3225.6626204223417`, certified `890420.383607373`.

---

On the small two-predictor dataset already used by the broad local test, the correct standard errors are approximately `[0.52915, 0.19626, 0.27217]`; the exact mutation produces `[1.63043, 0.88607, 1.05579]`. A frozen assertion against the pre-mutation values would plainly turn red without Longley-class conditioning. The existing test survives because its oracle is weak, not because the dataset cannot discriminate the mutation.
