# Sand Slabs

The parser accepts nonnegative machine-sized decimal coordinates, with positive
Z, exactly three coordinates per endpoint and one `~` separator per line. It
normalizes reversed endpoints, accepts LF or CRLF line endings, and rejects empty
input, blank records, non-line bricks and initial intersections. Bricks may be
single cubes or straight segments parallel to one axis.

Bricks settle in ascending original bottom height. A brick lands one level above
the highest settled top whose inclusive XY footprint overlaps its own. Only
bricks tied at that top support it. Reverse traversal and a height cutoff reduce
unnecessary footprint checks without affecting correctness. Valid starting scenes
only move downward; checked landing arithmetic also guards this calculation.

Part one counts bricks whose children all have another supporter. Part two removes
each brick in turn, using remaining-support counts and a FIFO queue to propagate
falls. A brick is queued once, when its last supporter is removed; the initial
removal is excluded from the count. Counts use checked machine-sized arithmetic.

Initial validation and settling take O(n²) time; settling is independent of tower
height. One cascade takes O(n + e) time and O(n) temporary space for n bricks and
e support edges, giving O(n(n + e)) time over all removals. The stored support
graph uses O(n + e) space.

Synthetic tests compare settling against unit-step falls and every removal in
256 generated scenes against voxel occupancy resettling. Additional cases cover
shared supports, rejoining cascade paths, reversed endpoints and invalid geometry.
Private cached fixtures remain external to the repository.
