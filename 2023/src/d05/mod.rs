use std::ops::Range;
use svutils::SolveError;

struct Mapping {
    source: Range<i64>,
    destination_start: i64,
}

impl Mapping {
    fn translate(&self, value: i64) -> Result<i64, SolveError> {
        value
            .checked_sub(self.source.start)
            .and_then(|offset| self.destination_start.checked_add(offset))
            .ok_or(SolveError::ArithmeticOverflow)
    }

    fn convert(&self, value: i64) -> Result<Option<i64>, SolveError> {
        if self.source.contains(&value) {
            Ok(Some(self.translate(value)?))
        } else {
            Ok(None)
        }
    }
}

fn convert(maps: &[Vec<Mapping>], mut num: i64) -> Result<i64, SolveError> {
    for map in maps {
        for mapping in map {
            if let Some(converted) = mapping.convert(num)? {
                num = converted;
                break;
            }
        }
    }
    Ok(num)
}

fn parse_seeds(input: &str) -> Result<Vec<i64>, SolveError> {
    let line = input.lines().next().ok_or(SolveError::EmptyInput)?;
    let numbers = line
        .strip_prefix("seeds:")
        .ok_or_else(|| SolveError::InvalidInput("expected 'seeds:' header".into()))?;
    let seeds = numbers
        .split_whitespace()
        .map(str::parse::<i64>)
        .collect::<Result<Vec<_>, _>>()?;
    if seeds.is_empty() {
        return Err(SolveError::InvalidInput("seed list is empty".into()));
    }
    Ok(seeds)
}

struct Almanac {
    seeds: Vec<i64>,
    maps: Vec<Vec<Mapping>>,
}

fn parse_mapping_row(line: &str) -> Result<Mapping, SolveError> {
    let values = line
        .split_whitespace()
        .map(str::parse::<i64>)
        .collect::<Result<Vec<_>, _>>()?;
    let [destination, source, length] = values.as_slice() else {
        return Err(SolveError::InvalidInput(
            "mapping row must contain destination, source, and length".into(),
        ));
    };
    if *length <= 0 {
        return Err(SolveError::InvalidInput(
            "mapping length must be positive".into(),
        ));
    }
    let end = source
        .checked_add(*length)
        .ok_or(SolveError::ArithmeticOverflow)?;
    Ok(Mapping {
        source: *source..end,
        destination_start: *destination,
    })
}

fn parse_almanac(input: &str) -> Result<Almanac, SolveError> {
    let seeds = parse_seeds(input)?;
    let mut lines = input.lines().skip(1).peekable();
    let mut maps = Vec::new();
    while let Some(line) = lines.next() {
        if line.trim().is_empty() {
            continue;
        }
        if !line.trim_end().ends_with(" map:") {
            return Err(SolveError::InvalidInput(
                "expected mapping header ending in ' map:'".into(),
            ));
        }
        let mut map = Vec::new();
        while let Some(line) = lines.next_if(|line| !line.trim().is_empty()) {
            map.push(parse_mapping_row(line)?);
        }
        maps.push(map);
    }
    Ok(Almanac { seeds, maps })
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let almanac = parse_almanac(input)?;
    let mut lowest = i64::MAX;
    for seed in almanac.seeds {
        lowest = lowest.min(convert(&almanac.maps, seed)?);
    }
    Ok(lowest.to_string())
}

fn convert_ranges(
    stage: &[Mapping],
    ranges: Vec<Range<i64>>,
) -> Result<Vec<Range<i64>>, SolveError> {
    let mut output = Vec::new();
    for range in ranges {
        let mut pending = vec![range];
        for mapping in stage {
            let mut remaining = Vec::new();
            for fragment in pending {
                let start = fragment.start.max(mapping.source.start);
                let end = fragment.end.min(mapping.source.end);
                if start >= end {
                    remaining.push(fragment);
                    continue;
                }
                if fragment.start < start {
                    remaining.push(fragment.start..start);
                }
                if end < fragment.end {
                    remaining.push(end..fragment.end);
                }
                // Matched pieces leave this stage; only unmatched pieces try later rows.
                output.push(mapping.translate(start)?..mapping.translate(end)?);
            }
            pending = remaining;
            if pending.is_empty() {
                break;
            }
        }
        output.extend(pending);
    }
    Ok(output)
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let Almanac { seeds, maps } = parse_almanac(input)?;
    let pairs = seeds.chunks_exact(2);
    if !pairs.remainder().is_empty() {
        return Err(SolveError::InvalidInput(
            "seed ranges require start/length pairs".into(),
        ));
    }
    let mut ranges = Vec::new();
    for pair in pairs {
        if pair[1] <= 0 {
            return Err(SolveError::InvalidInput(
                "seed range length must be positive".into(),
            ));
        }
        let end = pair[0]
            .checked_add(pair[1])
            .ok_or(SolveError::ArithmeticOverflow)?;
        ranges.push(pair[0]..end);
    }
    for stage in &maps {
        ranges = convert_ranges(stage, ranges)?;
    }
    let lowest = ranges
        .iter()
        .map(|range| range.start)
        .min()
        .ok_or(SolveError::EmptyInput)?;
    Ok(lowest.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 5;

    #[test]
    fn mapping_lengths_must_be_positive() {
        for length in [0, -1] {
            let input = format!("seeds: 12 1\n\nseed-to-location map:\n100 12 {length}");
            for solver in [part1, part2] {
                assert!(matches!(solver(&input), Err(SolveError::InvalidInput(_))));
            }
        }
    }

    #[test]
    fn translation_subtracts_source_before_adding_destination() {
        let start = i64::MAX - 2;
        let input = format!("seeds: {start} 1\n\nseed-to-location map:\n{start} {start} 1");
        assert_eq!(part1(&input).unwrap(), "1");
        assert_eq!(part2(&input).unwrap(), start.to_string());
        let input = format!("seeds: {start}\n\nseed-to-location map:\n{start} {start} 1");
        assert_eq!(part1(&input).unwrap(), start.to_string());
    }

    #[test]
    fn unrepresentable_translation_returns_an_error_in_both_parts() {
        let input = format!("seeds: 1 1\n\nseed-to-location map:\n{} 0 2", i64::MAX);
        for solver in [part1, part2] {
            assert!(matches!(
                solver(&input),
                Err(SolveError::ArithmeticOverflow)
            ));
        }
    }

    #[test]
    fn minimum_can_follow_a_mapping_end_in_an_unchanged_gap() {
        let input = "seeds: 7 8\n\nseed-to-location map:\n15 5 3\n22 10 3";
        assert_eq!(part2(input).unwrap(), "8");
    }

    #[test]
    fn a_matched_piece_skips_later_rows_but_enters_the_next_stage() {
        let stage = "seeds: 13 1\n\nseed-to-soil map:\n100 12 2\n200 13 1\n0 100 2";
        assert_eq!(part2(stage).unwrap(), "101");
        let input = format!("{stage}\n\nsoil-to-location map:\n0 100 2");
        assert_eq!(part2(&input).unwrap(), "1");
    }

    #[test]
    fn range_splitting_matches_individual_conversion_on_small_intervals() {
        let mapping_text = "\n\nseed-to-soil map:\n3 10 5\n3 20 5\n30 12 8\n\nsoil-to-location map:\n0 5 2\n20 0 3";
        for start in 0..30 {
            for length in 1..10 {
                let input = format!("seeds: {start} {length}{mapping_text}");
                let almanac = parse_almanac(&input).unwrap();
                let expected = (start..start + length)
                    .map(|seed| convert(&almanac.maps, seed).unwrap())
                    .min()
                    .unwrap();
                assert_eq!(part2(&input).unwrap(), expected.to_string());
            }
        }
    }

    #[test]
    fn seed_ranges_reject_nonpositive_lengths_and_overflow() {
        for input in ["seeds: 12 0", "seeds: 12 -1"] {
            assert!(matches!(part2(input), Err(SolveError::InvalidInput(_))));
        }
        let input = format!("seeds: {} 2", i64::MAX);
        assert!(matches!(part2(&input), Err(SolveError::ArithmeticOverflow)));
        let input = format!("seeds: 0 1\n\nseed-to-location map:\n{} 0 1", i64::MAX);
        assert!(matches!(part2(&input), Err(SolveError::ArithmeticOverflow)));
    }

    #[test]
    fn mapping_rows_require_exactly_three_numbers() {
        for rows in ["100 12", "100 12 5 6", "100 12\n5 200 34 2"] {
            let input = format!("seeds: 12 1\n\nseed-to-location map:\n{rows}");
            for solver in [part1, part2] {
                assert!(matches!(solver(&input), Err(SolveError::InvalidInput(_))));
            }
        }
    }

    #[test]
    fn malformed_mapping_numbers_return_errors() {
        let input = "seeds: 12 1\n\nseed-to-location map:\n100 nope 5";
        for solver in [part1, part2] {
            assert!(matches!(solver(input), Err(SolveError::InvalidNumber(_))));
        }
    }

    #[test]
    fn mapping_rows_preserve_boundaries_with_windows_line_endings() {
        let input = "seeds: 12 1\r\n\r\nseed-to-location map:\r\n100 12 1\r\n200 1 1\r\n";
        assert_eq!(part1(input).unwrap(), "100");
        assert_eq!(part2(input).unwrap(), "100");
    }

    #[test]
    fn individual_seeds_do_not_require_complete_range_pairs() {
        let input = "seeds: 12 34 56\n\nseed-to-location map:\n100 12 1";
        assert_eq!(part1(input).unwrap(), "34");
        assert!(matches!(part2(input), Err(SolveError::InvalidInput(_))));
    }

    #[test]
    fn malformed_seed_numbers_are_returned_as_errors() {
        for solver in [part1, part2] {
            assert!(matches!(
                solver("seeds: 12 oops 34"),
                Err(SolveError::InvalidNumber(_))
            ));
        }
    }

    #[test]
    fn seed_header_and_nonempty_list_are_required() {
        for input in ["seed: 12 34", "12 34", "seeds:", "seeds:   "] {
            assert!(matches!(
                parse_seeds(input),
                Err(SolveError::InvalidInput(_))
            ));
        }
        assert!(matches!(parse_seeds(""), Err(SolveError::EmptyInput)));
        assert_eq!(parse_seeds("seeds: 12  34").unwrap(), vec![12, 34]);
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
}
