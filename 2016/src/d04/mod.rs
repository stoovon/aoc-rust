use regex::Regex;
use std::cmp::Reverse;
use svutils::SolveError;

fn checksum_matches(name: &str, checksum: &str) -> bool {
    let mut counts = [0usize; 26];
    for letter in name.bytes().filter(u8::is_ascii_lowercase) {
        counts[usize::from(letter - b'a')] += 1;
    }

    let mut indices: [usize; 26] = std::array::from_fn(|index| index);
    indices.sort_unstable_by_key(|&index| (Reverse(counts[index]), index));
    indices
        .into_iter()
        .take_while(|&index| counts[index] != 0)
        .take(5)
        .map(|index| b'a' + index as u8)
        .eq(checksum.bytes())
}

struct Room<'a> {
    name: &'a str,
    sector: u64,
}

impl Room<'_> {
    fn decrypts_to(&self, target: &str) -> bool {
        let shift = (self.sector % 26) as u8;
        self.name
            .bytes()
            .map(|byte| match byte {
                b'-' => b' ',
                letter => b'a' + (letter - b'a' + shift) % 26,
            })
            .eq(target.bytes())
    }
}

fn real_rooms(
    input: &str,
) -> Result<impl Iterator<Item = Result<Room<'_>, SolveError>>, SolveError> {
    if input.is_empty() {
        return Err(SolveError::EmptyInput);
    }

    let pattern = Regex::new(r"\A([a-z]+(?:-[a-z]+)*)-([0-9]+)\[([a-z]{5})\]\z")
        .expect("room pattern is valid");
    Ok(input.lines().enumerate().filter_map(move |(index, line)| {
        let room = (|| {
            let captures = pattern.captures(line).ok_or_else(|| {
                SolveError::InvalidInput("expected encrypted-name-sector[checksum]".into())
            })?;
            let sector = captures[2].parse::<u64>()?;
            let name = captures.get(1).unwrap().as_str();
            Ok(checksum_matches(name, &captures[3]).then_some(Room { name, sector }))
        })();
        room.map_err(|error: SolveError| error.at_line(index + 1))
            .transpose()
    }))
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let mut sum = 0u64;
    for room in real_rooms(input)? {
        sum = sum
            .checked_add(room?.sector)
            .ok_or(SolveError::ArithmeticOverflow)?;
    }
    Ok(sum.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let mut sector = None;
    for room in real_rooms(input)? {
        let room = room?;
        if room.decrypts_to("northpole object storage") {
            sector = Some(room.sector);
        }
    }
    sector
        .map(|sector| sector.to_string())
        .ok_or(SolveError::NoSolution("northpole object storage room"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_orders_letters_and_skips_absent_letters() {
        assert!(!checksum_matches("zzyy-xwvu", "zyuvw"));
        assert!(checksum_matches("zzyy-xwvu", "yzuvw"));
        assert!(!checksum_matches("ccdd-eeff", "cdefa"));
    }

    #[test]
    fn skips_well_formed_decoys() {
        let input = "zzyy-xwvu-11[zyuvw]\r\nzzyy-xwvu-3[yzuvw]";
        assert_eq!(part1(input).unwrap(), "3");
        assert!(matches!(part2(input), Err(SolveError::NoSolution(_))));
    }

    #[test]
    fn rejects_malformed_rows() {
        for row in [
            " abcde-7[abcde]",
            "abcde-7[abcde]junk",
            "ab--cde-7[abcde]",
            "abcde-7[Abcde]",
            "abcde-7[abcde",
            "abcde-18446744073709551616[abced]",
        ] {
            let input = format!("abcde-1[abcde]\n{row}");
            assert!(matches!(
                part1(&input),
                Err(SolveError::AtLine { line: 2, .. })
            ));
        }
    }

    #[test]
    fn each_part_only_does_its_own_work() {
        let input = "abcde-18446744073709551615[abcde]\nmnqsgonkd-naidbs-rsnqzfd-1[ndsqa]";
        assert!(matches!(part1(input), Err(SolveError::ArithmeticOverflow)));
        assert_eq!(part2(input).unwrap(), "1");
        let malformed = format!("{input}\ntrailing junk");
        assert!(matches!(
            part2(&malformed),
            Err(SolveError::AtLine { line: 3, .. })
        ));
    }
}
