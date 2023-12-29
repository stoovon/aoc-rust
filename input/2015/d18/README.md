[<<<](../README.md)

# 2023::18 Like GIF For Your Yard

## Rubric summary

- Rules:
  - A light which is on stays on when 2 or 3 neighbors are on, and turns off otherwise.
  - A light which is off turns on if exactly 3 neighbors are on, and stays off otherwise.
- Example is 6 x 6, times 4 steps, yields 4 lights.
- 1. 100 x 100, times 100 steps, how many lights on?

## Input files

These can be downloaded and put into e.g.

| Input         | Yields                                     |
|---------------|--------------------------------------------|
| `example.txt` | `example-spec.1.txt`, `example-spec.2.txt` |
| `input.txt`   | `input-spec.1.txt`, `input-spec.2.txt`     |

## Solution Sketch / Solving Notes

- State Machine (game of life?)
- Points on grid.
  - Chebyshev distance to pick neighbours.
  - Apply simple rules.