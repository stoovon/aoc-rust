# Day 20 solver

Input consists of blank-separated tiles with positive, unique decimal IDs and
exactly ten rows of ten ASCII `.` or `#` cells. The tile count must be a nonempty
perfect square. CRLF and trailing blank lines are accepted; malformed records
return errors with line context.

Canonical border encodings ignore reversal and index tile positions. Repeated
sides of the same tile do not count as neighbours. Actual placement compares
oriented borders exactly.

Part 1 requires exactly four tiles with exactly two adjacent unmatched borders.
Those tiles are forced corners; their IDs are multiplied with checked arithmetic.
This contract permits the corner product without assembling the image. Inputs
whose outer borders have coincidental matches may not satisfy this contract and
return an error; part 1 does not fall back to assembly.

Part 2 uses an explicit backtracking stack, trying unused tiles in distinct
orientations and matching both existing neighbours. When exactly four forced
corners are identified, the first is anchored at the top left; global rotation
makes this safe, and both compatible orientations remain candidates. Otherwise,
all starting tiles remain available. Unmatched sides cannot face
the interior; outer sides may have coincidental matches. The first complete
arrangement is used. Different valid arrangements are not compared for roughness.
Assembly supports at most 1,024 tiles and 1,000,000 work units: one per indexed
candidate occurrence and one per orientation checked, including rejected choices.
Exhaustion returns a resource-limit error, distinct from an exhausted search that
proves no arrangement exists. The limits bound orientation storage and search
choices; they are not claims about puzzle solvability.

Rendering extracts each tile's eight-by-eight interior once. The image stays
fixed while the monster's coordinate pattern is rotated and reflected through
eight orientations. A reusable boolean mask counts each covered pixel once,
including overlaps. Roughness subtracts the greatest coverage in a single
orientation from the image's total marked pixels. No match leaves that total
unchanged. Both final valid row and column positions are included in the scan.
