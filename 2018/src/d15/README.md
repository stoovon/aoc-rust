# Day 15: Beverage Bandits

Maps must be nonempty rectangular ASCII grids containing only `.`, `#`, `E` and
`G`, with at least one elf and one goblin. Open edges are supported. Units start
with 200 HP. Malformed cells and rows report their line number.

Each independent battle borrows immutable terrain and copies the initial units.
A unit's vector index remains its ID after movement or death. Indexed occupancy
is cleared immediately on death. Reading order is fixed at the start of each
round; dead IDs are skipped. Only complete rounds count toward the checked score.

Movement uses reusable breadth-first search buffers. It examines the whole nearest
layer to select the earliest destination in reading order. FIFO traversal through
reading-ordered neighbours propagates the earliest first step among shortest
routes to each cell. Destination order takes priority over first-step order.
Attacks select the adjacent enemy with lowest HP, then reading order.

Part 1 runs one battle at attack power 3. Part 2 tries powers 3 through 200 in
increasing order, stopping an attempt immediately when an elf dies. Power 200
kills every goblin in one hit; all higher powers produce identical battles.
The search does not assume that casualty-free victory is monotonic in power.
If no power succeeds, it returns an error.

A round with no movement or attacks is a stalemate. Between attacks, positions
by stable ID detect exact repeated round-boundary states: HP and the surviving
roster remain fixed, attack powers are fixed for the attempt, and initiative is
derived from positions. Any attack clears that history. Part 1 reports a
stalemate; Part 2 skips that unsuccessful attempt.

Maps support at most 1,048,576 cells. Each attempt permits 100,000,000 work units:
one per initial-unit scan slot at each round, one per scheduled turn, and one per
map cell for each movement search. Cycle history retains at most 1,048,576 unit
positions. These are resource limits, not assumptions about victory. Exceeding a
limit returns an error and stops the search rather than skipping an unknown result
or reporting a partial score.
