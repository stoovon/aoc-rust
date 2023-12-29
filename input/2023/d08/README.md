[<<<](../README.md)

# 2023::08 Haunted Wasteland

## Rubric summary

## Input files

These can be downloaded and put into e.g.

| Input         | Yields                                     |
|---------------|--------------------------------------------|
| `example.txt` | `example-spec.1.txt`, `example-spec.2.txt` |
| `input.txt`   | `input-spec.1.txt`, `input-spec.2.txt`     |

## Solution Sketch / Solving Notes

- Probably my favourite problem and solution for Week 1.
- See https://www.reddit.com/r/adventofcode/comments/18did3d/2023_day_8_part_1_my_input_maze_plotted_using/.
  - The period of each cycle is the [LCM](https://en.wikipedia.org/wiki/Least_common_multiple) of the length 
    of the directions with the length of the cycle.
  - BFS from each node to find the length of each cycle.
    - Part 1 just considers node `AAA` -> `ZZZ`.
    - Part 2 is the combined LCMs, which can be done via:
        `lcm(a, b, c) = lcm(lcm(a, b), c)`