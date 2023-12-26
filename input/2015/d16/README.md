[<<<](../README.md)

# 2023::16 Aunt Sue

## Rubric summary

Given an input (multiple candidates) and a list of properties (held by the right candidate, but not by the others), find the matching Aunt Sue.

## Input files

These can be downloaded and put into e.g.

| Input         | Yields                                     |
|---------------|--------------------------------------------|
| `example.txt` | `example-spec.1.txt`, `example-spec.2.txt` |
| `input.txt`.  | `input-spec.1.txt`, `input-spec.2.txt`     |

## Solution Sketch / Solving Notes

Hashmap?
 - One map per value.
 - Put Aunt Sues into buckets based on property e.g. Cats 1, 2, 3, 4 and so on.
 - Multiple Aunt Sues can go into a bucket.
 - For each property in the list, retrieve all the buckets without the properties e.g. Cats 1-6, 8-Inf (we want cats 7).
   - Those Aunt Sues can be positively disqualified, so clear them in a bitmap / set.
   - If an Aunt Sue had that value but you didn't remember, she won't be in the list, but that's OK.
 - The survivor after all this elimination will be the matching Aunt Sue.
 - This is roughly similar to give a seed a fertilizer.