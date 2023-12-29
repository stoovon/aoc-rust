[<<<](../README.md)

# 2020::01 Report Repair

## Rubric summary

- Find the product of the two entries in a list that sum to 2020.
- Then, find the product of the three entries in a list that sum to 2020.

## Input files

These can be downloaded and put into e.g.

| Input         | Yields                                     |
|---------------|--------------------------------------------|
| `example.txt` | `example-spec.1.txt`, `example-spec.2.txt` |
| `input.txt`   | `input-spec.1.txt`, `input-spec.2.txt`     |

## Solution Sketch / Solving Notes

- Use `itertools::tuple_combinations` for a simple and effective solution that's easy to understand.
- Feels like we could push it further (not least, combine the two solutions by making the map function variadic), but a reasonable implementation in my view.