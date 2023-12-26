[<<<](../README.md)

# 2023::05 If You Give A Seed A Fertilizer

## Rubric summary

## Input files

These can be downloaded and put into e.g.

| Input         | Yields                                     |
|---------------|--------------------------------------------|
| `example.txt` | `example-spec.1.txt`, `example-spec.2.txt` |
| `input.txt`.  | `input-spec.1.txt`, `input-spec.2.txt`     |

## Solution Sketch / Solving Notes

- Got burned hard on Part 2. Be careful about growth / optimisation. Biggest lesson of Week 1.
- I heard people compare this to [Reactor Reboot](../../../2021/src/d22/README.md) and [Lanternfish](../../../2021/src/d06/README.md).
- Still a lot of meat on this, I can think of ways to further improve the solution chosen.
  - The solution ultimately included here is based very closely on 
    https://github.com/tumdum/aoc2023/blob/main/src/day05.rs, which made the most sense to me after looking at a number of better solutions than my original effort (which included lots of irrelevant structs). I may try to rewrite later.