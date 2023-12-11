[<<<](../../README.md)

# 2023::10 Pipe Maze

## Rubric summary

## Input files

These can be downloaded and put into `example.txt`, `example-spec.txt`, `input.txt` and `input-spec.txt`.

The expectation is that the spec files will contain a single number, which is the expected answer.

## Solution Sketch / Solving Notes

- Use [Shoelace formula](https://en.wikipedia.org/wiki/Shoelace_formula) and [Pick's theorem](https://en.wikipedia.org/wiki/Pick%27s_theorem)

[Inspiration](https://www.reddit.com/r/adventofcode/comments/18evyu9/comment/kcqmhwk/))
> Part 2: If we consider the closed loop as an integral polygon then Pick's theorem relates the area of the closed loop (which can be calculated using the shoelace formula), the number of integer points on the boundary of the closed loop (which is just the length of the close loop), and the number of integer points in the interior of the loop (which is the answer).

- My original approach was written in Python, but I studied several Rust approaches during the day and this is the version 
  which makes most sense to me. I learned A LOT about some of the operator overloads in Rust on this one.