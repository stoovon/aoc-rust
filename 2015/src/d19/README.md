# Day 19: Medicine for Rudolph

Input consists of one or more replacement rules, one blank separator and one target
molecule. Rule sides and the target are nonempty ASCII alphanumeric strings. Exact
duplicate rules, malformed records and extra target lines are rejected, with line
context where applicable. Different rules may share either side.

Replacements act on literal character substrings, including overlapping positions
and matches inside apparent element names. The solver does not impose chemical
word boundaries. Part 1 creates each one-step result once for distinct counting.
Rules and the target borrow input text; generated molecules own their contents.

Part 2 finds the minimum number of steps from `e`. It first attempts to derive an
additive character counter: every rule must increase the counter by the same
positive amount D. A derivation's length is therefore the target counter difference
divided by D. The counter does not establish reachability: a real reverse path to
`e` is also required before a fast-found result is accepted. Deterministic passes
through length-ordered rules provide the initial candidate path.

The counter is derived by checked integer elimination, with row GCD normalization
and a common denominator. Every original rule is then checked independently.
There is no floating point, assumed element-count formula or randomized restart.
The proof attempt supports up to 256 rules and 64 distinct characters; failure or
arithmetic limits decline that proof and permit exact search.

If the candidate path fails or no counter is available, breadth-first reverse
search finds a shortest path with value-based deduplication. `Rc<str>` shares
molecule contents between the queue and visited set. A valid counter can bound
the search depth. Cycles and reverse growth are allowed within resource limits.

Each search phase has a 10,000,000 potential-offset-check budget. The exact search
retains at most 65,536 distinct molecules and 32 MiB of molecule contents; the
candidate phase bounds its current molecule to 32 MiB. Budget exhaustion returns
an error, not a partial step count or an unreachability claim. General rewriting
can still have combinatorial growth. Answer counts and size arithmetic are checked.
