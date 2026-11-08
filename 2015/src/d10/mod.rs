use std::collections::HashMap;
use svutils::SolveError;

const MAX_ROUNDS: usize = 50;
const PREFIX_BYTES: usize = 64;
const MAX_PIECE_BYTES: usize = 256;
const MAX_PIECES: usize = 512;
const MAX_CACHE_BYTES: usize = 256 * 1024;
const MAX_WORK: usize = 20_000_000;

struct Budget {
    remaining: usize,
}
impl Budget {
    fn spend(&mut self, amount: usize) -> Result<(), SolveError> {
        self.remaining = self
            .remaining
            .checked_sub(amount)
            .ok_or(SolveError::ResourceLimit(
                "look-and-say work budget exceeded",
            ))?;
        Ok(())
    }
}

fn describe(input: &[u8], output: &mut Vec<u8>) {
    output.clear();
    let mut start = 0;
    while start < input.len() {
        let digit = input[start];
        let mut end = start + 1;
        while end < input.len() && input[end] == digit {
            end += 1;
        }
        let mut count = end - start;
        let mut digits = [0; 20];
        let mut offset = digits.len();
        while count > 0 {
            offset -= 1;
            digits[offset] = b'0' + (count % 10) as u8;
            count /= 10;
        }
        output.extend_from_slice(&digits[offset..]);
        output.push(digit);
        start = end;
    }
}

fn safe_boundary(
    last: u8,
    right: &[u8],
    rounds: usize,
    budget: &mut Budget,
) -> Result<bool, SolveError> {
    // The left suffix digit is invariant; the right prefix must stay different.
    let mut prefix = right[..right.len().min(PREFIX_BYTES)].to_vec();
    let mut complete = right.len() <= PREFIX_BYTES;
    let mut next = Vec::with_capacity(PREFIX_BYTES * 2);
    for round in 0..rounds {
        if prefix.first().is_none_or(|&first| first == last) {
            return Ok(false);
        }
        if round + 1 == rounds {
            return Ok(true);
        }
        budget.spend(prefix.len())?;
        let mut end = prefix.len();
        if !complete {
            // The truncated final run has an unknown length and cannot be described.
            let digit = prefix[end - 1];
            while end > 0 && prefix[end - 1] == digit {
                end -= 1;
            }
        }
        describe(&prefix[..end], &mut next);
        complete = complete && next.len() <= PREFIX_BYTES;
        next.truncate(PREFIX_BYTES);
        std::mem::swap(&mut prefix, &mut next);
    }
    Ok(true)
}

struct Piece {
    digits: Vec<u8>,
    children: Option<Vec<usize>>,
}
struct Chemistry {
    pieces: Vec<Piece>,
    indices: HashMap<Vec<u8>, usize>,
    cache_bytes: usize,
    budget: Budget,
}
impl Chemistry {
    fn new() -> Self {
        Self {
            pieces: Vec::new(),
            indices: HashMap::new(),
            cache_bytes: 0,
            budget: Budget {
                remaining: MAX_WORK,
            },
        }
    }
    fn charge_cache(&mut self, bytes: usize) -> Result<(), SolveError> {
        self.cache_bytes = self
            .cache_bytes
            .checked_add(bytes)
            .ok_or(SolveError::ArithmeticOverflow)?;
        if self.cache_bytes > MAX_CACHE_BYTES {
            return Err(SolveError::ResourceLimit(
                "look-and-say cache budget exceeded",
            ));
        }
        Ok(())
    }
    fn intern(&mut self, digits: &[u8]) -> Result<usize, SolveError> {
        if let Some(&index) = self.indices.get(digits) {
            return Ok(index);
        }
        if digits.len() > MAX_PIECE_BYTES || self.pieces.len() == MAX_PIECES {
            return Err(SolveError::ResourceLimit(
                "look-and-say piece budget exceeded",
            ));
        }
        self.charge_cache(2 * digits.len())?;
        let index = self.pieces.len();
        self.indices.insert(digits.to_vec(), index);
        self.pieces.push(Piece {
            digits: digits.to_vec(),
            children: None,
        });
        Ok(index)
    }
    fn split(&mut self, digits: &[u8]) -> Result<Vec<usize>, SolveError> {
        let mut children = Vec::new();
        let mut start = 0;
        for boundary in 1..digits.len() {
            self.budget.spend(1)?;
            if digits[boundary - 1] != digits[boundary]
                && safe_boundary(
                    digits[boundary - 1],
                    &digits[boundary..],
                    MAX_ROUNDS,
                    &mut self.budget,
                )?
            {
                children.push(self.intern(&digits[start..boundary])?);
                start = boundary;
            }
        }
        children.push(self.intern(&digits[start..])?);
        Ok(children)
    }
    fn evolve(&mut self, seed: &[u8], rounds: usize) -> Result<u64, SolveError> {
        if rounds > MAX_ROUNDS {
            return Err(SolveError::ResourceLimit(
                "look-and-say round budget exceeded",
            ));
        }
        let mut counts = [0u64; MAX_PIECES];
        for index in self.split(seed)? {
            counts[index] = counts[index]
                .checked_add(1)
                .ok_or(SolveError::ArithmeticOverflow)?;
        }
        let mut output = Vec::with_capacity(2 * MAX_PIECE_BYTES);
        for _ in 0..rounds {
            let mut next = [0u64; MAX_PIECES];
            let count = self.pieces.len();
            for index in 0..count {
                let copies = counts[index];
                if copies == 0 {
                    continue;
                }
                if self.pieces[index].children.is_none() {
                    self.budget.spend(self.pieces[index].digits.len())?;
                    describe(&self.pieces[index].digits, &mut output);
                    let children = self.split(&output)?;
                    self.charge_cache(children.len() * std::mem::size_of::<usize>())?;
                    self.pieces[index].children = Some(children);
                }
                let children = self.pieces[index].children.as_ref().unwrap();
                self.budget.spend(children.len())?;
                for &child in children {
                    next[child] = next[child]
                        .checked_add(copies)
                        .ok_or(SolveError::ArithmeticOverflow)?;
                }
            }
            counts = next;
        }
        self.pieces
            .iter()
            .zip(counts)
            .try_fold(0u64, |total, (piece, copies)| {
                let contribution = copies
                    .checked_mul(piece.digits.len() as u64)
                    .ok_or(SolveError::ArithmeticOverflow)?;
                total
                    .checked_add(contribution)
                    .ok_or(SolveError::ArithmeticOverflow)
            })
    }
}
fn solve(input: &str, rounds: usize) -> Result<String, SolveError> {
    let seed = input.trim();
    if seed.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    if seed.len() > MAX_PIECE_BYTES {
        return Err(SolveError::ResourceLimit(
            "look-and-say seed budget exceeded",
        ));
    }
    if !seed.bytes().all(|digit| digit.is_ascii_digit()) {
        return Err(SolveError::InvalidInput("expected ASCII digits".into()));
    }
    Chemistry::new()
        .evolve(seed.as_bytes(), rounds)
        .map(|length| length.to_string())
}
pub fn part1(input: &str) -> Result<String, SolveError> {
    solve(input, 40)
}
pub fn part2(input: &str) -> Result<String, SolveError> {
    solve(input, 50)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;
    fn reference(input: &str) -> String {
        let mut chars = input.chars().peekable();
        let mut output = String::new();
        while let Some(digit) = chars.next() {
            let mut count = 1;
            while chars.peek() == Some(&digit) {
                chars.next();
                count += 1;
            }
            output.push_str(&count.to_string());
            output.push(digit);
        }
        output
    }
    #[test]
    fn generated_seeds_and_boundaries_match_direct_expansion() {
        for length in 1..=5 {
            for mut code in 0..5usize.pow(length) {
                let mut seed = String::new();
                for _ in 0..length {
                    seed.push(['0', '1', '2', '3', '7'][code % 5]);
                    code /= 5;
                }
                let mut expected = seed.clone();
                let mut chemistry = Chemistry::new();
                let indices = chemistry.split(seed.as_bytes()).unwrap();
                let mut pieces: Vec<String> = indices
                    .iter()
                    .map(|&index| {
                        String::from_utf8(chemistry.pieces[index].digits.clone()).unwrap()
                    })
                    .collect();
                for rounds in 1..=10 {
                    expected = reference(&expected);
                    pieces = pieces.iter().map(|piece| reference(piece)).collect();
                    assert_eq!(pieces.concat(), expected);
                    assert_eq!(solve(&seed, rounds).unwrap(), expected.len().to_string());
                }
            }
        }
    }
    #[test]
    fn truncated_prefix_certificate_and_limits() {
        for seed in ["7".repeat(100), "7".repeat(101), "01237".repeat(40)] {
            let mut direct = seed.clone();
            for rounds in 1..=10 {
                direct = reference(&direct);
                assert_eq!(solve(&seed, rounds).unwrap(), direct.len().to_string());
            }
        }
        let mut budget = Budget {
            remaining: MAX_WORK,
        };
        assert!(safe_boundary(b'3', b"2", 3, &mut budget).unwrap());
        assert!(!safe_boundary(b'3', b"2", 4, &mut budget).unwrap());
        assert!(!safe_boundary(b'1', &vec![b'7'; 100], 50, &mut budget).unwrap());
        for bad in ["", " ", "4x", "12 3", "４"] {
            assert!(part1(bad).is_err());
        }
        assert!(solve(&"7".repeat(257), 1).is_err());
        assert!(solve("7", 51).is_err());
        let mut chemistry = Chemistry::new();
        chemistry.budget.remaining = 0;
        assert!(chemistry.evolve(b"737", 1).is_err());
        let mut chemistry = Chemistry::new();
        chemistry.cache_bytes = MAX_CACHE_BYTES;
        assert!(chemistry.intern(b"7").is_err());
        let mut chemistry = Chemistry::new();
        for index in 0..MAX_PIECES {
            chemistry.intern(index.to_string().as_bytes()).unwrap();
        }
        assert!(chemistry.intern(b"extra").is_err());
    }
    #[test]
    fn full_horizon_matches_direct_expansion() {
        for seed in ["707", "2307"] {
            let mut direct = seed.to_owned();
            for rounds in 1..=MAX_ROUNDS {
                direct = reference(&direct);
                assert_eq!(solve(seed, rounds).unwrap(), direct.len().to_string());
            }
        }
        assert_eq!(solve(&"7".repeat(51), 0).unwrap(), "51");
        assert!(matches!(
            solve("707", 51),
            Err(SolveError::ResourceLimit(_))
        ));
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 10, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 10, "input.txt", "input-spec.2.txt", part2);
    }
}
