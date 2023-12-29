extern crate core;

struct Password {
    pwd: Vec<char>,
    loops: usize,
}

impl Password {
    #![allow(dead_code)]
    fn new(pwd: &str) -> Self {
        Self {
            pwd: pwd.chars().collect(),
            loops: 0,
        }
    }

    fn next(&mut self) {
        for current in self.pwd.iter_mut().rev() {
            if *current == 'z' {
                *current = 'a';
            } else {
                *current = (*current as u8 + 1) as char;
                break;
            }
        }
        self.loops += 1;
    }

    fn is_valid(&self) -> bool {
        // Passwords must include one increasing straight of at least three
        // letters, like abc, bcd, cde, and so on, up to xyz. They cannot skip
        // letters; abd doesn't count.
        let mut straigh = false;
        for i in 0..self.pwd.len() - 2 {
            if self.pwd[i] as u8 + 1 == self.pwd[i + 1] as u8
                && self.pwd[i + 1] as u8 + 1 == self.pwd[i + 2] as u8
            {
                straigh = true;
            }
        }
        if !straigh {
            return false;
        }

        // Passwords may not contain the letters i, o, or l, as these letters
        // can be mistaken for other characters and are therefore confusing.
        for c in &self.pwd {
            match c {
                'i' | 'o' | 'l' => return false,
                _ => (),
            }
        }

        // Passwords must contain at least two different, non-overlapping pairs
        // of letters, like aa, bb, or zz.
        for i in 0..self.pwd.len() - 1 {
            // find a pair
            if self.pwd[i] == self.pwd[i + 1] {
                for j in i + 2..self.pwd.len() - 1 {
                    // then a second pair
                    if self.pwd[j] == self.pwd[j + 1] {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn next_valid(&mut self) {
        self.next();
        while !self.is_valid() {
            self.next();
        }
    }
}

pub fn fn1(data: &str) -> String {
    let mut pwd: Password = Password::new(data);
    pwd.next_valid();
    pwd.pwd.iter().collect()
}

pub fn fn2(data: &str) -> String {
    let mut pwd: Password = Password::new(data);
    pwd.next_valid();
    pwd.next_valid();
    pwd.pwd.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_string;

    const YEAR: i16 = 2015;
    const DAY: i16 = 11;

    #[test]
    fn test_fn1_example() {
        scaffold_test_string(YEAR, DAY, "example.txt", "example-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn1_input() {
        scaffold_test_string(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_example() {
        scaffold_test_string(YEAR, DAY, "example.txt", "example-spec.2.txt", fn2);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test_string(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
