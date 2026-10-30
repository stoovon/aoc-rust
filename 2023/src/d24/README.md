# Never Tell Me The Odds

Both parts share integer parsing and a three-dimensional hailstone model. Records
contain exactly three signed decimal coordinates per vector and one `@` separator.
Whitespace around coordinates and LF/CRLF line endings are accepted. Empty input,
blank records, decimal fractions, nonfinite values and extra fields are rejected
with source-line context. Integers and answers have arbitrary precision.

## Projected path crossings

Part one ignores Z. For two XY velocities `u` and `v`, let `d` be the difference
between their starting positions. The determinant `cross(u, v)` gives the common
denominator of their two intersection times; their numerators are `cross(d, v)`
and `cross(d, u)`. Normalize the denominator's sign, reject negative times, then
compare scaled integer intersection coordinates with the inclusive square bounds.
No floating point, division by an individual velocity, or per-pair fractions are
needed. Time zero is included. The two hailstones may reach the XY crossing at
different times.

Stationary XY paths are treated as points. Distinct parallel lines do not cross.
Collinear moving paths return an error because they do not define a unique line
intersection; this solver does not choose a counting convention for overlapping
rays. This limitation is explicit rather than silently treating them as disjoint.

## Rock trajectory

Part two solves for six values: the rock's starting position `r` and velocity `v`.
For hailstone position `p` and velocity `u`, a collision requires
`(p - r) × (v - u) = 0`. Subtracting the equation for an anchor hailstone cancels
the shared nonlinear `r × v` term. The XY, XZ and YZ components yield linear rows.
Exact rational Gaussian elimination retains at most six independent pivot rows.
Redundant rows are harmless; contradictions and insufficient linear rank are
reported as errors. No guessed velocity range is used.

Once six pivots determine a candidate, verify every original hailstone in all
three coordinates at a single nonnegative time. This verification is essential:
a solution of the subtracted equations need not satisfy the originals. Relative
velocity components may be zero; identical trajectories are valid. Collision
times may be exact fractions, while the rock's position and velocity must be
integral. Return the exact sum of its starting coordinates. Rank-deficient linear
systems are rejected rather than attempting a separate nonlinear search.

Ignoring integer bit complexity, part one takes O(n²) pair checks. Part two takes
O(n) work with a fixed six-column linear system and full-candidate verification.
Parsing stores O(n) hailstones; elimination stores only six rows.

## Verification

Independently constructed crossing cases cover vertical and stationary paths,
time direction, boundaries and very large coordinates. Generated known-rock
scenes include fractional collision times and velocities beyond the old search
range. Other tests cover singular prefixes, contradictions, input permutations,
nonintegral rocks, past collisions, and a full-rank linear solution that fails the
original collision equations. No puzzle inputs or official examples are embedded.

## Rust arithmetic

Use `num-bigint` and `num-rational` for exact integers and fractions. Part one's
usual path uses checked `i128` arithmetic on precomputed XY projections. Any
unrepresentable input or intermediate, or degenerate geometry, falls back to the
same arbitrary-precision calculation. Part two uses `BigRational` throughout.
Overflow regressions compare the fallback with the arbitrary-precision reference.
