# Day 13: Knights of the Dinner Table

Both parts require complete directed happiness data: exactly one entry for each
ordered pair of distinct guests. Duplicate, missing, self-referential and malformed
entries are rejected. Names are nonempty ASCII letters, digits or underscores;
happiness magnitudes are nonnegative decimal integers producing signed i64 values.

The parser combines each pair's directed values into a symmetric score matrix.
Part 2 consumes that matrix and adds an unnamed neutral guest, avoiding collisions
with real names. The shared search borrows the resulting matrix.

The search fixes one guest to remove rotational duplicates. Its subset table
stores the best score for each set of seated guests and last guest, then adds the
closing connection to the fixed guest. `None` denotes an unreachable state;
negative scores, including i64::MIN, remain legitimate values.

Time is O(n² × 2ⁿ) and table storage is O(n × 2ⁿ), with no retained permutations.
The table budget is 1,048,576 entries: at most 17 total guests, including the neutral
guest when present. Larger tables return an error without an exhaustive fallback.
Pair, path and closing sums use checked i64 arithmetic and report overflow.
