use svutils::SolveError;

const MAX_HOUSES: usize = 8_000_000;
const MAX_DELIVERIES: usize = 100_000_000;

#[derive(Clone, Copy)]
enum Delivery {
    Unlimited,
    FirstFifty,
}

impl Delivery {
    fn presents_per_elf(self) -> u64 {
        match self {
            Self::Unlimited => 10,
            Self::FirstFifty => 11,
        }
    }

    fn visit_limit(self) -> Option<usize> {
        match self {
            Self::Unlimited => None,
            Self::FirstFifty => Some(50),
        }
    }
}

fn parse(input: &str) -> Result<u64, SolveError> {
    let digits = input.trim();
    if digits.is_empty() {
        return Err(SolveError::EmptyInput);
    }
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SolveError::InvalidInput(
            "expected a positive decimal present threshold".into(),
        ));
    }
    let threshold = digits.parse()?;
    if threshold == 0 {
        return Err(SolveError::InvalidInput(
            "present threshold must be positive".into(),
        ));
    }
    Ok(threshold)
}

fn capped_total(total: u64, contribution: u64, threshold: u64) -> u64 {
    // Stored totals are at most threshold, so this addition cannot overflow.
    total + contribution.min(threshold - total)
}

fn distribute(
    last_house: usize,
    threshold: u64,
    delivery: Delivery,
) -> Result<Vec<u64>, SolveError> {
    if last_house == 0 || last_house > MAX_HOUSES {
        return Err(SolveError::ResourceLimit(
            "present table house budget exceeded",
        ));
    }
    let visit_numbers = delivery
        .visit_limit()
        .map_or(last_house, |limit| last_house.min(limit));
    // Each power-of-two group of visit numbers contributes at most last_house deliveries.
    let groups = (usize::BITS - visit_numbers.leading_zeros()) as usize;
    last_house
        .checked_mul(groups)
        .filter(|&work| work <= MAX_DELIVERIES)
        .ok_or(SolveError::ResourceLimit(
            "present delivery work budget exceeded",
        ))?;
    let slots = last_house
        .checked_add(1)
        .ok_or(SolveError::ResourceLimit("present table size overflow"))?;
    let mut houses = Vec::new();
    houses
        .try_reserve_exact(slots)
        .map_err(|_| SolveError::ResourceLimit("could not allocate present table"))?;
    houses.resize(slots, 0u64);
    let multiplier = delivery.presents_per_elf();
    let limit = delivery.visit_limit().unwrap_or(last_house);
    for elf in 1..=last_house {
        let presents = (elf as u64)
            .checked_mul(multiplier)
            .ok_or(SolveError::ArithmeticOverflow)?;
        let mut house = elf;
        let mut delivered = 0;
        while house <= last_house && delivered < limit {
            houses[house] = capped_total(houses[house], presents, threshold);
            house += elf;
            delivered += 1;
        }
    }
    Ok(houses)
}

fn first_house(threshold: u64, delivery: Delivery) -> Result<usize, SolveError> {
    let multiplier = delivery.presents_per_elf();
    // A house receives its own elf's contribution even with the fifty-visit limit.
    let upper = threshold / multiplier + u64::from(threshold % multiplier != 0);
    let upper = usize::try_from(upper)
        .map_err(|_| SolveError::ResourceLimit("present table does not fit this platform"))?;
    distribute(upper, threshold, delivery)?
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(house, &presents)| (presents >= threshold).then_some(house))
        .ok_or_else(|| {
            SolveError::InvalidInput("no qualifying house within the proven bound".into())
        })
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    first_house(parse(input)?, Delivery::Unlimited).map(|house| house.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    first_house(parse(input)?, Delivery::FirstFifty).map(|house| house.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    fn divisor_presents(house: usize, limited: bool) -> u128 {
        let sum: u128 = (1..=house)
            .filter(|&elf| house % elf == 0 && (!limited || house / elf <= 50))
            .map(|elf| elf as u128)
            .sum();
        sum * if limited { 11 } else { 10 }
    }

    #[test]
    fn sieve_matches_independent_divisor_sums_and_caps() {
        for (delivery, limited) in [(Delivery::Unlimited, false), (Delivery::FirstFifty, true)] {
            let exact = distribute(400, u64::MAX, delivery).unwrap();
            let capped = distribute(400, 173, delivery).unwrap();
            for house in 1..=400 {
                let expected = divisor_presents(house, limited);
                assert_eq!(u128::from(exact[house]), expected);
                assert_eq!(u128::from(capped[house]), expected.min(173));
            }
        }
        let limited = distribute(51, u64::MAX, Delivery::FirstFifty).unwrap();
        assert_eq!(limited[50], 1023);
        assert_eq!(limited[51], 781);
    }

    #[test]
    fn first_qualifying_house_matches_direct_search() {
        for threshold in 1..=1000 {
            for (delivery, limited) in [(Delivery::Unlimited, false), (Delivery::FirstFifty, true)]
            {
                let expected = (1..=threshold as usize)
                    .find(|&house| divisor_presents(house, limited) >= u128::from(threshold))
                    .unwrap();
                assert_eq!(first_house(threshold, delivery).unwrap(), expected);
            }
        }
        for solver in [part1, part2] {
            assert_eq!(solver("17").unwrap(), "2");
            assert_eq!(solver("5").unwrap(), "1");
        }
    }

    #[test]
    fn threshold_capping_handles_u64_overflow() {
        assert_eq!(capped_total(12, 99, 20), 20);
        assert_eq!(capped_total(0, u64::MAX, 20), 20);
        assert_eq!(capped_total(u64::MAX - 1, 20, u64::MAX), u64::MAX);
        assert_eq!(capped_total(20, 99, 20), 20);
    }

    #[test]
    fn capped_addition_matches_wide_arithmetic() {
        for threshold in [0, 1, 20, u64::MAX] {
            for total in [0, threshold / 2, threshold] {
                for contribution in [0, 1, 19, u64::MAX] {
                    let expected =
                        (u128::from(total) + u128::from(contribution)).min(u128::from(threshold));
                    assert_eq!(
                        u128::from(capped_total(total, contribution, threshold)),
                        expected
                    );
                }
            }
        }
    }

    #[test]
    fn parser_and_resource_limits_return_errors() {
        for solver in [part1, part2] {
            for empty in ["", " \r\n"] {
                assert!(matches!(solver(empty), Err(SolveError::EmptyInput)));
            }
            for bad in [
                "0",
                "-17",
                "+17",
                "17.0",
                "17 23",
                "１７",
                "18446744073709551616",
            ] {
                assert!(solver(bad).is_err(), "{bad:?}");
            }
            assert_eq!(solver(" \t17\r\n").unwrap(), "2");
            assert!(matches!(
                solver("18446744073709551615"),
                Err(SolveError::ResourceLimit(_))
            ));
        }
        assert!(matches!(
            distribute(MAX_HOUSES + 1, 17, Delivery::FirstFifty),
            Err(SolveError::ResourceLimit(_))
        ));
        assert!(matches!(
            distribute(MAX_HOUSES, 17, Delivery::Unlimited),
            Err(SolveError::ResourceLimit(_))
        ));
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 20, "input.txt", "input-spec.1.txt", part1);
    }
    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 20, "input.txt", "input-spec.2.txt", part2);
    }
}
