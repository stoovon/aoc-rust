extern crate core;

use md5;

fn probable_match(digest: &md5::Digest) -> bool {
    digest[0] == 0 && digest[1] == 0 && digest[2] & 0xf0 == 0
}

pub fn fn1(input: &str) -> String {
	let mut password = String::new();
	for i in 0.. {
		let digest = md5::compute(format!("{}{}", input, i));

		if !probable_match(&digest) {
			continue;
		}

		let digest_str = format!("{:x}", digest);
		if digest_str.chars().take(5).all(|c| c == '0') {
			password.push(digest_str.chars().nth(5).unwrap());

			if password.len() == 8 {
				break;
			}
		}
	}

    password
}

pub fn fn2(input: &str) -> String {
	let mut password : Vec<char> = vec![' '; 8];
	for i in 0.. {
		let digest = md5::compute(format!("{}{}", input, i));

		if !probable_match(&digest) {
			continue;
		}

		let digest_str = format!("{:x}", digest);
		let digest_arr = digest_str.chars().collect::<Vec<char>>();
		if digest_arr.iter().take(5).all(|c| c == &'0') {
			let slot = digest_arr[5].to_digit(10).unwrap_or(9) as usize;
			if slot < 8 && password[slot] == ' ' {
				password[slot] = digest_arr[6];
			}

			if password.iter().all(|c| c != &' ') {
				break;
			}
		}
	}

    password.into_iter().collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_string;

    const YEAR: i16 = 2016;
    const DAY: i16 = 5;

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
