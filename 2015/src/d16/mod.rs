extern crate core;

use std::str::FromStr;

struct AuntSue {
    number: i64,
    children: Option<i64>,
    cats: Option<i64>,
    samoyeds: Option<i64>,
    pomeranians: Option<i64>,
    akitas: Option<i64>,
    vizslas: Option<i64>,
    goldfish: Option<i64>,
    trees: Option<i64>,
    cars: Option<i64>,
    perfumes: Option<i64>,
}

impl FromStr for AuntSue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut children = None;
        let mut cats = None;
        let mut samoyeds = None;
        let mut pomeranians = None;
        let mut akitas = None;
        let mut vizslas = None;
        let mut goldfish = None;
        let mut trees = None;
        let mut cars = None;
        let mut perfumes = None;

        let parts: Vec<&str> = s.splitn(2, ':').collect();
        let head = parts[0].trim();

        let number = Some(head.trim_start_matches("Sue ").parse().unwrap());

        let tail = parts[1].trim();

        for (_, part) in tail.split(", ").enumerate() {
            let mut parts = part.split(": ");
            let key = parts.next().unwrap();
            let value = parts.next().unwrap();

            match key {
                "children" => children = Some(value.parse().unwrap()),
                "cats" => cats = Some(value.parse().unwrap()),
                "samoyeds" => samoyeds = Some(value.parse().unwrap()),
                "pomeranians" => pomeranians = Some(value.parse().unwrap()),
                "akitas" => akitas = Some(value.parse().unwrap()),
                "vizslas" => vizslas = Some(value.parse().unwrap()),
                "goldfish" => goldfish = Some(value.parse().unwrap()),
                "trees" => trees = Some(value.parse().unwrap()),
                "cars" => cars = Some(value.parse().unwrap()),
                "perfumes" => perfumes = Some(value.parse().unwrap()),
                _ => panic!("Unknown key: {}", key),
            }
        }

        Ok(Self {
            number: number.unwrap(),
            children,
            cats,
            samoyeds,
            pomeranians,
            akitas,
            vizslas,
            goldfish,
            trees,
            cars,
            perfumes,
        })
    }
}

fn default_sue() -> AuntSue {
    AuntSue {
        number: 0,
        children: Some(3),
        cats: Some(7),
        samoyeds: Some(2),
        pomeranians: Some(3),
        akitas: Some(0),
        vizslas: Some(0),
        goldfish: Some(5),
        trees: Some(3),
        cars: Some(2),
        perfumes: Some(1),
    }
}

fn eq(candidate_aunt_prop: &Option<i64>, default_aunt_prop: &Option<i64>) -> bool {
    if let Some(prop) = candidate_aunt_prop {
        return *prop == default_aunt_prop.unwrap();
    }

    true
}

fn leq(candidate_aunt_prop: &Option<i64>, default_aunt_prop: &Option<i64>) -> bool {
    if let Some(prop) = candidate_aunt_prop {
        return *prop <= default_aunt_prop.unwrap()
    }

    true
}

fn gt(candidate_aunt_prop: &Option<i64>, default_aunt_prop: &Option<i64>) -> bool {
    if let Some(prop) = candidate_aunt_prop {
        return *prop > default_aunt_prop.unwrap()
    }

    true
}

pub fn fn1(input: &str) -> i64 {
    let aunts = input
        .lines()
        .map(|line| line.parse::<AuntSue>().unwrap())
        .collect::<Vec<_>>();

    let default_sue = default_sue();

    for aunt in aunts {
        let matches = eq(&aunt.children, &default_sue.children)
            && eq(&aunt.cats, &default_sue.cats)
            && eq(&aunt.samoyeds, &default_sue.samoyeds)
            && eq(&aunt.pomeranians, &default_sue.pomeranians)
            && eq(&aunt.akitas, &default_sue.akitas)
            && eq(&aunt.vizslas, &default_sue.vizslas)
            && eq(&aunt.goldfish, &default_sue.goldfish)
            && eq(&aunt.trees, &default_sue.trees)
            && eq(&aunt.cars, &default_sue.cars)
            && eq(&aunt.perfumes, &default_sue.perfumes);

        if matches {
            return aunt.number;
        }
    }

    0
}

pub fn fn2(input: &str) -> i64 {
    let aunts = input
        .lines()
        .map(|line| line.parse::<AuntSue>().unwrap())
        .collect::<Vec<_>>();
    let default_sue = AuntSue {
        number: 0,
        children: Some(3),
        cats: Some(7),
        samoyeds: Some(2),
        pomeranians: Some(3),
        akitas: Some(0),
        vizslas: Some(0),
        goldfish: Some(5),
        trees: Some(3),
        cars: Some(2),
        perfumes: Some(1),
    };

    for aunt in aunts {
        let matches = eq(&aunt.children, &default_sue.children)
        && gt(&aunt.cats, &default_sue.cats)
        && eq(&aunt.samoyeds, &default_sue.samoyeds)
        && leq(&aunt.pomeranians, &default_sue.pomeranians)
        && eq(&aunt.akitas, &default_sue.akitas)
        && eq(&aunt.vizslas, &default_sue.vizslas)
        && leq(&aunt.goldfish, &default_sue.goldfish)
        && gt(&aunt.trees, &default_sue.trees)
        && eq(&aunt.cars, &default_sue.cars)
        && eq(&aunt.perfumes, &default_sue.perfumes);

        if matches {
            return aunt.number;
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 16;

    #[test]
    fn test_fn1_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.1.txt", fn1);
    }

    #[test]
    fn test_fn2_input() {
        scaffold_test(YEAR, DAY, "input.txt", "input-spec.2.txt", fn2);
    }
}
