extern crate core;

#[derive(Debug)]
struct Parcel {
    w: i64,
    d: i64,
    h: i64,
}

fn parse_parcel(s: &str) -> Option<Parcel> {
    let mut parts: Vec<i64> = s.split('x').filter_map(|part| part.parse().ok()).collect();
    parts.sort_unstable();
    if parts.len() == 3 {
        Some(Parcel {
            w: parts[0],
            d: parts[1],
            h: parts[2],
        })
    } else {
        None
    }
}

fn parse_the_parcels(input: &str) -> Vec<Parcel> {
    input
        .lines()
        .map(|l| parse_parcel(l))
        .filter_map(|p| p)
        .collect()
}

pub fn fn1(input: &str) -> i64 {
    parse_the_parcels(input)
        .iter()
        .map(|parcel| {
            2 * (parcel.w * parcel.d + parcel.w * parcel.h + parcel.d * parcel.h)
                + parcel.w * parcel.d
        })
        .sum()
}

pub fn fn2(input: &str) -> i64 {
    parse_the_parcels(input)
        .iter()
        .map(|parcel| {
            // Potentially gnarly, let's try this
            2 * (parcel.w + parcel.d) +
            // Cubic volume
            parcel.w * parcel.d * parcel.h
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 2;

    #[test]
    fn test_fn1_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_example() {
        scaffold_test(YEAR, DAY, "example.txt", "example-spec.2.txt", fn2);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
