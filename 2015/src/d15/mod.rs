extern crate core;

use std::collections::HashMap;
use itertools::Itertools;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Attributes {
    capacity: i64,
    durability: i64,
    flavor: i64,
    texture: i64,
    calories: i64,
}

impl Attributes {
    fn new(capacity: i64, durability: i64, flavor: i64, texture: i64, calories: i64) -> Self {
        Self {
            capacity,
            durability,
            flavor,
            texture,
            calories,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Ingredient {
    name: String,
    attributes: Attributes,
}

struct Store {
    ingredients: Vec<Ingredient>,
}

impl Store {
    fn best_recipe(&self, exact_desired_calories: i64) -> i64 {
        let mut best_score = 0;

        for combination in self.ingredients.iter().combinations_with_replacement(100) {
            let recipe = combination.iter().copied()
            .fold(HashMap::new(), |mut map:HashMap<Ingredient, i64>, val|{
                map.entry(val.clone())
                   .and_modify(|frq|*frq+=1)
                   .or_insert(1);
                map
            });

            let mut capacity = 0;
            let mut durability = 0;
            let mut flavor = 0;
            let mut texture = 0;
            let mut calories = 0;

            for (ingredient, frq) in recipe {
                capacity += ingredient.attributes.capacity * frq;
                durability += ingredient.attributes.durability * frq;
                flavor += ingredient.attributes.flavor * frq;
                texture += ingredient.attributes.texture * frq;
                calories += ingredient.attributes.calories * frq;
            }
    
            let score = capacity.max(0) * durability.max(0) * flavor.max(0) * texture.max(0);

            if exact_desired_calories != -1 && calories != exact_desired_calories {
                continue;
            }
    
            if score > best_score {
                best_score = score;
            }
        }

        best_score
    }
}

impl FromStr for Store {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ingredients = Vec::new();

        for line in s.lines() {
            let mut parts = line.split(": ");
            let name = parts.next().unwrap();
            let mut attributes = Vec::new();

            for part in parts {
                let mut parts = part.split(", ");
                let capacity = parts.next().unwrap().split(" ").nth(1).unwrap().parse().unwrap();
                let durability = parts.next().unwrap().split(" ").nth(1).unwrap().parse().unwrap();
                let flavor = parts.next().unwrap().split(" ").nth(1).unwrap().parse().unwrap();
                let texture = parts.next().unwrap().split(" ").nth(1).unwrap().parse().unwrap();
                let calories = parts.next().unwrap().split(" ").nth(1).unwrap().parse().unwrap();

                attributes.push(Attributes::new(capacity, durability, flavor, texture, calories));
            }

            ingredients.push(Ingredient {
                name: name.to_string(),
                attributes: attributes[0].clone(),
            });
        }

        Ok(Self { ingredients })
    }
}

pub fn fn1(input: &str) -> i64 {
    let store: Store = input.parse().unwrap();

    store.best_recipe(-1)
}

pub fn fn2(input: &str) -> i64 {
    let store: Store = input.parse().unwrap();

    store.best_recipe(500)
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2015;
    const DAY: i16 = 15;

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
