extern crate core;

fn parse_patterns(input: &str) -> Vec<(Vec<u64>, Vec<u64>)> {
    input
        .split("\n\n")
        .map(|pattern| {
            let lines: Vec<&str> = pattern.lines().collect();
            let rows = lines
                .iter()
                .map(|line| {
                    line.chars().fold(0u64, |mut row, c| {
                        row <<= 1;
                        if c == '#' {
                            row |= 1;
                        }
                        row
                    })
                })
                .collect::<Vec<u64>>();
            let mut cols = vec![0u64; lines[0].len()];
            for (_, &row) in rows.iter().enumerate() {
                for (j, col) in cols.iter_mut().enumerate() {
                    *col <<= 1;
                    *col |= (row >> (lines[0].len() - j - 1)) & 1;
                }
            }
            (rows, cols)
        })
        .collect()
}

fn find_reflection(values: &[u64]) -> Option<usize> {
    // Input always seems to have an odd size (makes sense), so always skip first row or column.
    'outer: for (i, pair) in values.iter().zip(values.iter().skip(1)).enumerate() {
        if pair.0 != pair.1 {
            continue;
        }
        let mut reflection_pos = 1;
        while i as i32 - reflection_pos >= 0 && i as i32 + reflection_pos + 1 < values.len() as i32 {
            if values[i - reflection_pos as usize] != values[i + reflection_pos as usize + 1] {

                continue 'outer;
            }
            reflection_pos += 1;
        }
        return Some(i);
    }
    None
}

fn find_smudged_reflection(values: &[u64]) -> Option<usize> {
    'outer: for (i, pair) in values.iter().zip(values.iter().skip(1)).enumerate() {
        let mut smudge_seen = false;

        // If different and Hamming distance != 1
        if pair.0 != pair.1 && (pair.0 ^ pair.1).count_ones() != 1 {
            continue;
        } else if pair.0 != pair.1 {
            // If Hamming distance if 1 we have a match
            smudge_seen = true;
        }
        let mut reflection_pos = 1;
        while i as i32 - reflection_pos >= 0 && i as i32 + reflection_pos + 1 < values.len() as i32 {
            let left = values[i - reflection_pos as usize];
            let right = values[i + reflection_pos as usize + 1];
            // Hamming distance again.
            if left != right && (left ^ right).count_ones() != 1 {
                continue 'outer;
            } else if left != right {
                if smudge_seen {
                    continue 'outer;
                }
                smudge_seen = true;
            }
            reflection_pos += 1;
        }
        if smudge_seen {
            return Some(i);
        }
    }
    None
}

pub fn fn1(input: &str) -> i64 {
    parse_patterns(input)
        .iter()
        .map(|(rows, cols)| {
            if let Some(i) = find_reflection(rows) {
                100 * (i + 1)
            } else if let Some(i) = find_reflection(cols) {
                i + 1
            } else {
                unreachable!()
            }
        })
        .sum::<usize>() as i64
}

pub fn fn2(input: &str) -> i64 {
    parse_patterns(input)
        .iter()
        .map(|(rows, cols)| {
            if let Some(i) = find_smudged_reflection(rows) {
                100 * (i + 1)
            } else if let Some(i) = find_smudged_reflection(cols) {
                i + 1
            } else {
                unreachable!()
            }
        })
        .sum::<usize>() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::load_spec;

    #[test]
    fn test_fn1_example() {
        assert_eq!(fn1(include_str!("example.txt")), load_spec(include_str!("example-spec.1.txt")));
    }

    #[test]
    fn test_fn1_input() {
        assert_eq!(fn1(include_str!("input.txt")), load_spec(include_str!("input-spec.1.txt")));
    }

    #[test]
    fn test_fn2_example() {
        assert_eq!(fn2(include_str!("example.txt")), load_spec(include_str!("example-spec.2.txt")));
    }

    #[test]
    fn test_fn2_input() {
        assert_eq!(fn2(include_str!("input.txt")), load_spec(include_str!("input-spec.2.txt")));
    }
}