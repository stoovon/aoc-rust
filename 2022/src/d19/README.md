# Day 19 solver contracts

Blueprints have positive, unique `i64` IDs and six nonnegative `u16` costs.
The complete record grammar is validated, including resource names, punctuation,
and absence of trailing data. CRLF is accepted. Empty input, malformed rows,
duplicate IDs, and numeric range errors return errors with line context.
Zero-cost recipes are supported.

Part 1 sums checked ID-weighted maxima over all blueprints at 24 minutes. Part 2
multiplies maxima at 32 minutes for the first three available blueprints; one or
two records are supported. Every record is validated before either composition.
Both parts share a search from one ore robot and zero resources.

A build must be affordable before collection. Existing robots collect during that
minute; the new robot starts producing the following minute. Search states are
small Copy values. Minutes and robot counts use `u8`; balances and the best
geode result use `u16`. With at most one build per minute and a horizon at most
32, robot counts are at most 33, ore at most 528, and other balances at most 496.
Recipe costs use `u16` independently of the reachable balance bounds.

The depth-first search tries geode robots first but still explores other legal
investments. Forcing an affordable geode robot can exclude a better result;
a synthetic 14-minute regression demonstrates the distinction.

Safe pruning retains an optimistic bound allowing one new geode robot every
remaining minute, production caps at maximum per-minute spending, and elimination
of waiting followed by an already-affordable build without intervening construction.
The last case is dominated by building earlier and collecting its extra output.
The search has no memoization table. A per-blueprint budget permits 10,000,000
state visits, including terminal and pruned visits. Exhaustion propagates a
resource-limit error; it never returns a partial best result. Recursion is bounded
by the supported horizon.

Tests contain independent synthetic data, including 6,561 comparisons against an
exhaustive minute-by-minute oracle without production pruning, and regressions for
balances above 255, malformed records, composition, arithmetic, and work limits.
The oracle's memoization is confined to tests. Puzzle inputs and personal expected
answers remain external local fixtures.
