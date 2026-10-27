use svutils::SolveError;

struct Pattern {
    rows: Vec<Vec<u64>>,
    columns: Vec<Vec<u64>>,
}

fn parse_pattern(lines: &[(usize, &str)]) -> Result<Pattern, SolveError> {
    let width = lines[0].1.len();
    let height = lines.len();
    let mut rows = vec![vec![0_u64; width.div_ceil(64)]; height];
    let mut columns = vec![vec![0_u64; height.div_ceil(64)]; width];
    for (y, &(line_number, line)) in lines.iter().enumerate() {
        let invalid = |message: &str| SolveError::InvalidInput(message.into()).at_line(line_number);
        if line.len() != width {
            return Err(invalid("pattern rows must have equal width"));
        }
        for (x, tile) in line.bytes().enumerate() {
            match tile {
                b'#' => {
                    rows[y][x / 64] |= 1_u64 << (x % 64);
                    columns[x][y / 64] |= 1_u64 << (y % 64);
                }
                b'.' => {}
                _ => return Err(invalid("pattern accepts only '.' and '#'")),
            }
        }
    }
    Ok(Pattern { rows, columns })
}

fn parse_patterns(input: &str) -> Result<Vec<Pattern>, SolveError> {
    if input.trim().is_empty() {
        return Err(SolveError::EmptyInput);
    }
    let mut patterns = Vec::new();
    let mut lines = Vec::new();
    for (index, line) in input.lines().enumerate() {
        if line.is_empty() {
            if !lines.is_empty() {
                patterns.push(parse_pattern(&lines)?);
                lines.clear();
            }
        } else {
            lines.push((index + 1, line));
        }
    }
    if !lines.is_empty() {
        patterns.push(parse_pattern(&lines)?);
    }
    Ok(patterns)
}

fn reflection_boundaries(
    values: &[Vec<u64>],
    required_mismatches: u32,
) -> impl Iterator<Item = usize> + '_ {
    (1..values.len()).filter(move |&boundary| {
        let mut remaining = required_mismatches;
        for (left, right) in values[..boundary].iter().rev().zip(&values[boundary..]) {
            for (&left_word, &right_word) in left.iter().zip(right) {
                let Some(budget) = remaining.checked_sub((left_word ^ right_word).count_ones())
                else {
                    return false;
                };
                remaining = budget;
            }
        }
        remaining == 0
    })
}

fn reflection_score(pattern: &Pattern, mismatches: u32) -> Result<i64, SolveError> {
    let mut candidates = reflection_boundaries(&pattern.rows, mismatches)
        .map(|boundary| (boundary, 100_i64))
        .chain(reflection_boundaries(&pattern.columns, mismatches).map(|boundary| (boundary, 1)));
    let (boundary, multiplier) = candidates
        .next()
        .ok_or_else(|| SolveError::InvalidInput("pattern has no qualifying reflection".into()))?;
    if candidates.next().is_some() {
        return Err(SolveError::InvalidInput(
            "pattern has multiple qualifying reflections".into(),
        ));
    }
    i64::try_from(boundary)
        .ok()
        .and_then(|boundary| boundary.checked_mul(multiplier))
        .ok_or(SolveError::ArithmeticOverflow)
}

fn solve(input: &str, mismatches: u32) -> Result<String, SolveError> {
    let patterns = parse_patterns(input)?;
    let total = patterns.iter().try_fold(0_i64, |total, pattern| {
        total
            .checked_add(reflection_score(pattern, mismatches)?)
            .ok_or(SolveError::ArithmeticOverflow)
    })?;
    Ok(total.to_string())
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    solve(input, 0)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    solve(input, 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    fn first_reflection(values: &[Vec<u64>], mismatches: u32) -> Option<usize> {
        reflection_boundaries(values, mismatches).next()
    }

    fn single_word_reflection(values: &[u64], mismatches: u32) -> Option<usize> {
        let packed: Vec<_> = values.iter().map(|&value| vec![value]).collect();
        first_reflection(&packed, mismatches)
    }

    const YEAR: i16 = 2023;
    const DAY: i16 = 13;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }

    #[test]
    fn reflection_requires_the_exact_mismatch_count() {
        assert_eq!(single_word_reflection(&[0b101, 0b101], 0), Some(1));
        assert_eq!(single_word_reflection(&[0b101, 0b101], 1), None);
        assert_eq!(single_word_reflection(&[0b101, 0b100], 0), None);
        assert_eq!(single_word_reflection(&[0b101, 0b100], 1), Some(1));
        assert_eq!(single_word_reflection(&[0b101, 0b010], 1), None);
    }

    #[test]
    fn mismatch_budget_applies_across_all_mirrored_pairs() {
        assert_eq!(
            single_word_reflection(&[0b11, 0b00, 0b00, 0b10], 1),
            Some(2)
        );
        assert_eq!(single_word_reflection(&[0b11, 0b00, 0b01, 0b10], 1), None);
    }

    #[test]
    fn reflection_compares_only_the_overlapping_sides() {
        assert_eq!(single_word_reflection(&[0b11, 0b11, 0b00], 0), Some(1));
        assert_eq!(single_word_reflection(&[0b00, 0b11, 0b11], 0), Some(2));
        assert_eq!(single_word_reflection(&[], 0), None);
        assert_eq!(single_word_reflection(&[0b11], 0), None);
    }

    #[test]
    fn packing_preserves_cells_across_word_boundaries() {
        for size in [64, 65, 129] {
            let mut row = vec![b'.'; size];
            for index in [0, 63, size - 1] {
                row[index] = b'#';
            }
            let row = String::from_utf8(row).unwrap();
            let pattern = parse_patterns(&format!("{row}\n{row}"))
                .unwrap()
                .pop()
                .unwrap();
            assert_eq!(first_reflection(&pattern.rows, 0), Some(1));
            for x in 0..size {
                let expected = matches!(x, 0 | 63) || x == size - 1;
                assert_eq!(pattern.rows[0][x / 64] & (1 << (x % 64)) != 0, expected);
                assert_eq!(pattern.columns[x][0], if expected { 3 } else { 0 });
            }
            let input = (0..size)
                .map(|y| if y == size - 1 { "##" } else { ".." })
                .collect::<Vec<_>>()
                .join("\n");
            let pattern = parse_patterns(&input).unwrap().pop().unwrap();
            assert_eq!(first_reflection(&pattern.columns, 0), Some(1));
            assert_eq!(pattern.columns[0][(size - 1) / 64], 1 << ((size - 1) % 64));
        }
    }

    #[test]
    fn smudges_in_later_words_are_counted() {
        assert_eq!(first_reflection(&[vec![0, 0], vec![0, 1]], 0), None);
        assert_eq!(first_reflection(&[vec![0, 0], vec![0, 1]], 1), Some(1));
        assert_eq!(first_reflection(&[vec![0, 0], vec![1, 1]], 1), None);
    }

    #[test]
    fn malformed_and_empty_documents_are_rejected() {
        for input in ["", " \t\r\n"] {
            assert!(matches!(part1(input), Err(SolveError::EmptyInput)));
        }
        for input in ["##\n.", "#x", "#é", "# ", "##\n \n##"] {
            assert!(part1(input).is_err());
            assert!(part2(input).is_err());
        }
    }

    #[test]
    fn exactly_one_reflection_is_required_across_both_orientations() {
        assert_eq!(part1("#\n#").unwrap(), "100");
        assert_eq!(part1("##").unwrap(), "1");
        assert_eq!(part2("#\n.").unwrap(), "100");
        assert_eq!(part2("#.").unwrap(), "1");
        for input in ["#", "#.\n.#"] {
            assert!(part1(input).is_err());
        }
        for input in ["##\n##", "#\n#\n#"] {
            assert!(part1(input).unwrap_err().to_string().contains("multiple"));
        }
        for input in ["##", "#.\n..", "#\n.\n#"] {
            assert!(part2(input).is_err());
        }
    }

    #[test]
    fn crlf_and_blank_line_separators_are_supported() {
        assert_eq!(part1("##\r\n\r\n#\r\n#\r\n").unwrap(), "101");
        assert_eq!(part1("\n##\n\n\n#\n#\n\n").unwrap(), "101");
    }

    #[test]
    fn packed_reflections_match_direct_cell_comparisons() {
        for height in 1..=3 {
            for width in 1..=3 {
                for mask in 0..(1_usize << (width * height)) {
                    let cells: Vec<Vec<_>> = (0..height)
                        .map(|y| {
                            (0..width)
                                .map(|x| {
                                    if mask & (1 << (y * width + x)) == 0 {
                                        b'.'
                                    } else {
                                        b'#'
                                    }
                                })
                                .collect()
                        })
                        .collect();
                    let input = cells
                        .iter()
                        .map(|row| std::str::from_utf8(row).unwrap())
                        .collect::<Vec<_>>()
                        .join("\n");
                    let pattern = parse_patterns(&input).unwrap().pop().unwrap();
                    for mismatches in [0, 1] {
                        for horizontal in [true, false] {
                            let length = if horizontal { height } else { width };
                            let across = if horizontal { width } else { height };
                            let expected: Vec<_> = (1..length)
                                .filter(|&boundary| {
                                    let mut differences = 0;
                                    for offset in 0..boundary.min(length - boundary) {
                                        let a = boundary - offset - 1;
                                        let b = boundary + offset;
                                        for position in 0..across {
                                            let (left, right) = if horizontal {
                                                (cells[a][position], cells[b][position])
                                            } else {
                                                (cells[position][a], cells[position][b])
                                            };
                                            differences += u32::from(left != right);
                                        }
                                    }
                                    differences == mismatches
                                })
                                .collect();
                            let packed = if horizontal {
                                &pattern.rows
                            } else {
                                &pattern.columns
                            };
                            assert_eq!(
                                reflection_boundaries(packed, mismatches).collect::<Vec<_>>(),
                                expected
                            );
                        }
                    }
                }
            }
        }
    }
}
