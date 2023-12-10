[<<<](../../README.md)

# 2020::01 Report Repair

## Rubric summary

- Find the product of the two entries in a list that sum to 2020.
- Then, find the product of the three entries in a list that sum to 2020.

## Input files

These can be downloaded and put into `example.txt`, `example-spec.txt`, `input.txt` and `input-spec.txt`.

The expectation is that the spec files will contain a single number, which is the expected answer.

## Solution Sketch / Solving Notes

- Use `itertools::tuple_combinations` for a simple and effective solution that's easy to understand.
- Feels like we could push it further (not least, combine the two solutions by making the map function variadic), but a reasonable implementation in my view.