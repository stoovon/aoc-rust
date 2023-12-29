extern crate core;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Step {
    Reverse,
    Cut(i128),
    Deal(i128),
}

fn parse(input: &str) -> Vec<Step> {
    let reverse_re: Regex = Regex::new(r"deal into new stack").unwrap();
    let cut_re: Regex = Regex::new(r"cut\s([-01234567890]+)").unwrap();
    let deal_re: Regex = Regex::new(r"deal with increment\s([01234567890]+)").unwrap();

    input.lines()
        .map(|val| {
            if reverse_re.is_match(val) {
                Step::Reverse
            } else if cut_re.is_match(val) {
                let caps = cut_re.captures(val).unwrap();
                let num = caps[1].parse::<i128>().unwrap();
                Step::Cut(num)
            } else {
                let caps = deal_re.captures(val).unwrap();
                let num = caps[1].parse::<i128>().unwrap();
                Step::Deal(num)
            }
        })
        .collect()
}

pub fn fn1(input: &str) -> i128 {
    // To solve requires knowledge of [modular arithemetic](https://en.wikipedia.org/wiki/Modular_arithmetic).

    // Basic modular identities:
    // * (a + b) mod m = (a mod m) + (b mod m)
    // * (a * b) mod m = (a mod m) * (b mod m)

    // TIL it's possible to use [modular inverse](https://en.wikipedia.org/wiki/Modular_multiplicative_inverse)
    // instead of division.

    // Each of the shuffle operations is a *linear congruence* of the form:
    // 
    // `Xₙ₊₁ = (aXₙ + c) mod m`
    //
    // "Deal into new stack" (reverses deck) can be represented as:
    // `Xₙ₊₁ = ((m - 1) * Xₙ + (m - 1)) mod m`
    //
    // "cut N cards" is:
    //
    // `Xₙ₊₁ = 1 * Xₙ + (m - N)) mod m`
    //
    // "deal with increment N" is:
    //
    // `Xₙ₊₁ = N * Xₙ + 0) mod m`
    //
    // Using knowledge of modular identities and how they apply to linear congruences (see e.g. Applied Cryptography), 
    // the shuffle operations can be combined into a single linear congruence using techniques such as:
    // `Xₙ₊₁ = a₂ * (a₁Xₙ + c₁) + c₂) mod m = (a₁a₂Xₙ + a₂c₁ + c₂) mod m`

    linear_congruence(input, 10007).shuffle(2019)
}

fn modular_exponentiation(number: i128, exponent: i128, modulus: i128) -> i128 {
    let mut b = number;
    let mut c = 1;
    let mut e = exponent;

    while e > 0 {
        if e & 1 == 1 {
            c = (c * b) % modulus;
        }
        b = (b * b) % modulus;
        e = e >> 1;
    }

    c
}

// Modular multiplicative inverse
fn modular_multiplicative_inverse(number: i128, modulus: i128) -> i128 {
    let mut t = 0;
    let mut newt = 1;
    let mut r = modulus;
    let mut newr = number;

    while newr != 0 {
        let quotient = r / newr;
        (t, newt) = (newt, t - quotient * newt);
        (r, newr) = (newr, r - quotient * newr);
    }

    if t < 0 {
        t = t + modulus;
    }
    t
}

struct ShuffleStep {
    a: i128,
    c: i128,
    m: i128,
}

impl ShuffleStep {
    fn compose(&self, other: &ShuffleStep) -> ShuffleStep {
        let m = self.m;
        let a = (self.a * other.a) % m;
        let c = (self.c * other.a + other.c) % m;
        ShuffleStep { a, c, m }
    }

    fn inverse(&self) -> ShuffleStep {
        let m = self.m;
        let a = modular_multiplicative_inverse(self.a, m);
        let c = m - (a * self.c) % m;
        ShuffleStep { a, c, m }
    }

    fn power(&self, e: i128) -> ShuffleStep {
        let m = self.m;
        let a = modular_exponentiation(self.a, e, m);
        let c = (((a - 1) * modular_multiplicative_inverse(self.a - 1, m) % m) * self.c) % m;
        ShuffleStep { a, c, m }
    }

    fn shuffle(&self, index: i128) -> i128 {
        (self.a * index + self.c) % self.m
    }
}

fn linear_congruence(input: &str, m: i128) -> ShuffleStep {
    parse(input)
        .into_iter()
        .map(|step| match step {
            Step::Reverse => ShuffleStep { a: m - 1, c: m - 1, m },
            Step::Cut(n) => {
                let n = (m - n % m) % m;
                ShuffleStep { a: 1, c: n, m }
            }
            Step::Deal(n) => {
                let n = (m + n % m) % m;
                ShuffleStep { a: n, c: 0, m }
            }
        })
        .reduce(|a, b| a.compose(&b))
        .unwrap()
}

pub fn fn2(input: &str) -> i128 {
    // To find the card that ends up at index 2020 we need to find an inverse congruence that then
    // can be applied to reverse things. When we're done, we'll end up with the identity congruence:
    //
    // `(a₁a₂Xₙ + a₂c₁ + c₂) mod m = (Xₙ + 0) mod m`
    //
    // This implies that `a₁a₂ mod m = 1` which is the definition of the modular inverse`a₂ = a₁⁻¹`.
    //
    // The constant term `(a₂c₁ + c₂) mod m = 0` implies `c₂ = m - a₂c₁`.
    //
    //
    // `cₙ = c(aⁿ - 1)((a - 1)⁻¹) mod m`
    //
    // `Xₙ₊₁ = (aⁿXₙ + c(aⁿ - 1)((a - 1)⁻¹)) mod m`
    //
    // If we then raise this to the Nth power we can solve:

    const DECK_LEN: i128 = 119_315_717_514_047;
    const SHUFFLE_COUNT: i128 = 101_741_582_076_661;

    linear_congruence(input, DECK_LEN).inverse().power(SHUFFLE_COUNT).shuffle(2020)
}

// deck(input, 10007).shuffle(2019)
// deck(input, 119315717514047).inverse().power(101741582076661).shuffle(2020)

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_wide;

    const YEAR: i16 = 2019;
    const DAY: i16 = 22;

    // #[test]
    // #[ignore]
    // fn test_fn1_example() {
    //     scaffold_test_wide(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    // }

    #[test]
    fn test_fn1_input() {
        scaffold_test_wide(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    // #[test]
    // #[ignore]
    // fn test_fn2_example() {
    //     scaffold_test_wide(YEAR, DAY, "example.txt", "example-spec.2.txt", fn2);
    // }

    #[test]
    fn test_fn2_input() {
        scaffold_test_wide(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
