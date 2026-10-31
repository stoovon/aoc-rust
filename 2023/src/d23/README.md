# A Long Walk

Accept a nonempty rectangular ASCII map containing `#`, `.`, `^`, `>`, `v`,
and `<`, with exactly one walkable opening in each of the top and bottom rows.
Discover those openings rather than assuming their columns. LF and CRLF line
endings are accepted. Reject blank rows, unknown tiles, ragged maps, and missing
or ambiguous boundary openings.

Both parts seek the longest entrance-to-exit route without revisiting a tile.
Part one follows the arrow when leaving a slope tile; entering a slope is not
restricted. Part two treats slopes as ordinary paths. A missing route is an
error; an entrance that is also the exit has a valid zero-step route.

Compress the map into endpoints and junctions with more than two walkable
neighbors. Trace each outgoing corridor independently, checking slope directions
along the way. Dead ends and routes back to the same junction cannot contribute
to a successful simple route. Between the same ordered pair of junctions, retain
the longest legal corridor: its interior has no branches, so replacing a shorter
parallel corridor cannot affect subsequent choices. Opposite directions are
handled independently.

Backtracking explores the resulting graph, marking junctions only for the
current path and stopping at the exit. If the exit has exactly one incoming
corridor, search instead ends at its predecessor and adds that mandatory
corridor length. The original exit is blocked during this search. Leaving the
predecessor elsewhere cannot produce a valid finish without revisiting it.
This condition is checked on the directed graph; multiple incoming corridors
use the ordinary search, and coincident endpoints still return zero.

Distinct undirected corridors have disjoint interiors, so a simple compressed
path corresponds to a simple tile path. Its total distance is bounded by the
number of map tiles minus one; distance arithmetic therefore fits the map index
type.
No search-state cache is retained.

For N tiles and J vertices, compression takes O(N + J log J) time and O(N + J)
space. Longest-simple-path search remains exponential in J in the worst case,
with O(J) additional space for visited flags and recursive calls. The solver
performs an exhaustive search without imposing a fixed search budget.

Synthetic tests cover parallel corridors with asymmetric slope permissions,
shifted endpoints, narrow maps, malformed input, unreachable exits, and zero-step
routes. Independent tile-level backtracking checks every one of 4,096 small wall
patterns, with and without slopes and under both slope policies: 16,384
comparisons. Puzzle datasets and personal expected answers remain external.
