# Day 21: Step Counter

Part 1 stays inside the original garden. It discovers each plot at its shortest
distance and accumulates counts for even and odd distances. A plot is reachable
in exactly N steps when its distance is at most N and has the same parity: spare
steps can be spent walking along an edge and back. An isolated start is handled
separately because it has no such edge.

Part 2 uses an infinitely repeating garden and requires a square map. It builds a
candidate distance model from a 3×3 block of map copies, then at most a 5×5 block.
Outside this core, the proposed distance adds one map width per outward tile
translation. The solver checks one surrounding ring, including boundaries
between copies.

The check establishes the shortest-distance equation, a unique zero at the start,
and closure of unreachable regions. Beyond the core, translations preserve the
neighbour-distance differences, so finite checks cover the infinite model.
Only a validated model becomes `CertifiedGarden`.

The final count partitions the map into the core, side strips and quadrants,
counting translated plots with exact parity-aware arithmetic. It does not fit a
polynomial or search out to the target step count.

Valid maps outside this supported distance model can be rejected. Failure to
validate within the fixed budget returns an error rather than starting an
unbounded search.
