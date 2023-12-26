[<<<](../README.md)

# 2023::10 Elves Look, Elves Say

## Rubric summary

## Input files

These can be downloaded and put into e.g.

| Input         | Yields                                     |
|---------------|--------------------------------------------|
| `example.txt` | `example-spec.1.txt`, `example-spec.2.txt` |
| `input.txt`.  | `input-spec.1.txt`, `input-spec.2.txt`     |

## Solution Sketch / Solving Notes

- Absolutely fascinating problem.
- Looked at several very interesting and powerful ideas in order to solve, and then looked at several solutions to polish.
  - [A brutally effective hash function in Rust](https://nnethercote.github.io/2021/12/08/a-brutally-effective-hash-function-in-rust.html)
  - [Firefox source code](https://searchfox.org/mozilla-central/rev/633345116df55e2d37be9be6555aa739656c5a7d/mfbt/HashFunctions.h#109-153)
- With sensible data structures, the solution was straightforward.
  - I don't think that we need to squeeze this much performance generally, but was fun to experiment.