[<<<](../README.md)

# 2023::17 No Such Thing as Too Much

## Rubric summary

- Bin packing
  1. Number of combinations to store 150 litres.

## Input files

These can be downloaded and put into e.g.

| Input         | Yields                                     |
|---------------|--------------------------------------------|
| `example.txt` | `example-spec.1.txt`, `example-spec.2.txt` |
| `input.txt`   | `input-spec.1.txt`, `input-spec.2.txt`     |

## Solution Sketch / Solving Notes

- Sort the inputs, highest bucket first.
- Can use the typical "make change with denominations" approach (modulo for the win), but of course it will need to have a stack and change shape depending on progress.
  - That'll find the greediest way, but could potentially be refined by burning each big bucket one by one and going again in order to move past local maximums.
    - In fact, we could repeat this approach as each pass is done, such that we do a BFS.