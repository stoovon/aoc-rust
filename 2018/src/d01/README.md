# Chronal Calibration

Both parts parse the complete input before solving. Each nonblank record contains
one signed decimal `i64` change; an optional `+` or `-`, surrounding whitespace,
and LF/CRLF line endings are accepted. Empty input, blank records, malformed
numbers and out-of-range changes return errors with source-line context. Invalid
later records cannot be hidden by an early repeat.

Part one sums one pass. Part two finds the first frequency visited twice, counting
the initial zero as visited. A first-pass check returns the earliest repeat within
that pass; zero total drift always repeats by its end.

## Finite repeat analysis

Let `S` be the total change per pass and `p[i]` the frequency before change `i`.
At phase `i` of cycle `k`, the frequency is `p[i] + k*S`. If the first pass has no
repeat, `S` is nonzero and its prefixes are distinct. Normalize negative drift by
negating all prefixes, making each cycle add `abs(S)`.

Only prefixes with the same Euclidean remainder modulo `abs(S)` can ever meet.
Sort prefixes by remainder and normalized frequency. Within a remainder group,
the lower prefix first reaches the next higher prefix after
`(higher - lower) / abs(S)` cycles. A more distant higher prefix cannot be reached
sooner. Adjacent pairs therefore suffice.

Compare candidates by `(cycle count, phase offset)` to select the earliest
chronological repeat. This avoids multiplying a potentially enormous cycle count
by the input length. Undo the normalization to recover the repeated frequency.
If no matching pair exists, report that the frequency never repeats. There is no
arbitrary iteration cap and no search whose work depends on the repeat time.

Part two takes O(n log n) time and O(n) space, ignoring integer bit complexity;
its first-pass set is bounded by the input length. Part one takes O(n) time and
parsing stores O(n) changes.

## Verification

Exhaustively compare with an independent step-by-step simulator for all 19,530
lists of one through six changes drawn from -2 through 2. The simulator's 200-step
bound exceeds the possible first-repeat time for this bounded input domain.
Additional synthetic cases cover no-repeat inputs, zero and negative drift,
first-pass ordering, trillion-cycle gaps, strict parsing and wide frequencies.
Puzzle inputs and expected personal answers remain in the external local cache.

## Rust numbers

Parse individual changes as `i64`; use checked `i128` for prefix frequencies,
normalization and cycle differences. On supported targets with at most 64-bit
`usize`, any addressable list of `i64` changes and any difference between its
prefixes fit `i128`. The checks also make failures explicit if those assumptions
change. Answers are returned as decimal strings, without narrowing to `i64`.
