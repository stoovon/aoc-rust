use std::collections::HashSet;
use svutils::SolveError;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct Attributes {
    capacity: i64,
    durability: i64,
    flavor: i64,
    texture: i64,
    calories: i64,
}

impl Attributes {
    fn add_scaled(self, ingredient: Self, teaspoons: usize) -> Result<Self, SolveError> {
        let amount = i64::try_from(teaspoons).map_err(|_| SolveError::ArithmeticOverflow)?;
        let add = |total: i64, value: i64| {
            value
                .checked_mul(amount)
                .and_then(|scaled| total.checked_add(scaled))
                .ok_or(SolveError::ArithmeticOverflow)
        };
        Ok(Self {
            capacity: add(self.capacity, ingredient.capacity)?,
            durability: add(self.durability, ingredient.durability)?,
            flavor: add(self.flavor, ingredient.flavor)?,
            texture: add(self.texture, ingredient.texture)?,
            calories: add(self.calories, ingredient.calories)?,
        })
    }

    fn score(self) -> Result<i64, SolveError> {
        let properties = [self.capacity, self.durability, self.flavor, self.texture];
        if properties.iter().any(|&value| value <= 0) {
            return Ok(0);
        }
        properties.into_iter().try_fold(1i64, |score, value| {
            score
                .checked_mul(value)
                .ok_or(SolveError::ArithmeticOverflow)
        })
    }
}

fn parse_record(line: &str) -> Result<(&str, Attributes), SolveError> {
    let invalid = || SolveError::InvalidInput("expected ingredient: property value, ...".into());
    let (name, fields) = line.split_once(": ").ok_or_else(invalid)?;
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(SolveError::InvalidInput(
            "expected a nonempty ingredient name".into(),
        ));
    }
    let (mut capacity, mut durability, mut flavor, mut texture, mut calories) =
        (None, None, None, None, None);
    for field in fields.split(", ") {
        let (property, value) = field.split_once(' ').ok_or_else(invalid)?;
        let slot = match property {
            "capacity" => &mut capacity,
            "durability" => &mut durability,
            "flavor" => &mut flavor,
            "texture" => &mut texture,
            "calories" => &mut calories,
            _ => {
                return Err(SolveError::InvalidInput(format!(
                    "unknown ingredient property '{property}'"
                )))
            }
        };
        if slot.replace(value.parse::<i64>()?).is_some() {
            return Err(SolveError::InvalidInput(format!(
                "duplicate ingredient property '{property}'"
            )));
        }
    }
    let (Some(capacity), Some(durability), Some(flavor), Some(texture), Some(calories)) =
        (capacity, durability, flavor, texture, calories)
    else {
        return Err(SolveError::InvalidInput(
            "expected capacity, durability, flavor, texture and calories exactly once".into(),
        ));
    };
    Ok((
        name,
        Attributes {
            capacity,
            durability,
            flavor,
            texture,
            calories,
        },
    ))
}

fn check_recipe_budget(ingredients: usize, teaspoons: usize) -> Result<(), SolveError> {
    if ingredients == 0 {
        return Err(SolveError::InvalidInput(
            "expected at least one ingredient".into(),
        ));
    }
    // Quantity assignments number C(teaspoons + ingredients - 1, ingredients - 1).
    let smaller = teaspoons.min(ingredients - 1);
    let larger = teaspoons.max(ingredients - 1) as u128;
    let unsupported =
        || SolveError::InvalidInput("recipe search exceeds the 10,000,000-recipe budget".into());
    let mut recipes = 1u128;
    for index in 1..=smaller {
        recipes = recipes
            .checked_mul(larger + index as u128)
            .ok_or_else(unsupported)?
            / index as u128;
        if recipes > 10_000_000 {
            return Err(unsupported());
        }
    }
    Ok(())
}

fn best_assignment(
    ingredients: &[Attributes],
    teaspoons: usize,
    totals: Attributes,
    required_calories: Option<i64>,
) -> Result<Option<i64>, SolveError> {
    let (ingredient, remaining) = ingredients
        .split_first()
        .ok_or_else(|| SolveError::InvalidInput("expected at least one ingredient".into()))?;
    if teaspoons == 0 || remaining.is_empty() {
        let totals = totals.add_scaled(*ingredient, teaspoons)?;
        if required_calories.is_some_and(|target| totals.calories != target) {
            return Ok(None);
        }
        return totals.score().map(Some);
    }
    let mut best = None;
    for amount in 0..=teaspoons {
        let totals = totals.add_scaled(*ingredient, amount)?;
        let Some(candidate) =
            best_assignment(remaining, teaspoons - amount, totals, required_calories)?
        else {
            continue;
        };
        best = Some(best.map_or(candidate, |score: i64| score.max(candidate)));
    }
    Ok(best)
}

struct Store {
    ingredients: Vec<Attributes>,
}

impl Store {
    fn parse(input: &str) -> Result<Self, SolveError> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let mut names = HashSet::new();
        let mut ingredients = Vec::new();
        for (index, line) in input.lines().enumerate() {
            let (name, attributes) =
                parse_record(line).map_err(|error| error.at_line(index + 1))?;
            if !names.insert(name) {
                return Err(
                    SolveError::InvalidInput(format!("duplicate ingredient '{name}'"))
                        .at_line(index + 1),
                );
            }
            ingredients.push(attributes);
        }
        Ok(Self { ingredients })
    }

    fn best_recipe(
        &self,
        teaspoons: usize,
        required_calories: Option<i64>,
    ) -> Result<i64, SolveError> {
        check_recipe_budget(self.ingredients.len(), teaspoons)?;
        best_assignment(
            &self.ingredients,
            teaspoons,
            Attributes::default(),
            required_calories,
        )?
        .ok_or_else(|| SolveError::InvalidInput("no recipe meets the calorie requirement".into()))
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    Store::parse(input)?
        .best_recipe(100, None)
        .map(|score| score.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    Store::parse(input)?
        .best_recipe(100, Some(500))
        .map(|score| score.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const MIXTURE: &str = "Grain: capacity 3, durability 1, flavor 1, texture 1, calories 4\nSpice: capacity 1, durability 3, flavor 1, texture 1, calories 6";
    const SINGLE: &str = "Sugar: capacity 1, durability 1, flavor 1, texture 1, calories 5";

    fn expanded_recipe_reference(
        ingredients: &[Attributes],
        teaspoons: usize,
        required: Option<i64>,
    ) -> Option<i64> {
        fn visit(
            ingredients: &[Attributes],
            recipe: &mut Vec<usize>,
            teaspoons: usize,
            start: usize,
            required: Option<i64>,
            best: &mut Option<i64>,
        ) {
            if recipe.len() == teaspoons {
                let mut totals = [0i128; 5];
                for &index in recipe.iter() {
                    let ingredient = ingredients[index];
                    for (total, value) in totals.iter_mut().zip([
                        ingredient.capacity,
                        ingredient.durability,
                        ingredient.flavor,
                        ingredient.texture,
                        ingredient.calories,
                    ]) {
                        *total += i128::from(value);
                    }
                }
                if required.is_some_and(|target| totals[4] != i128::from(target)) {
                    return;
                }
                let score: i128 = totals[..4].iter().map(|value| (*value).max(0)).product();
                let score = i64::try_from(score).unwrap();
                *best = Some(best.map_or(score, |old| old.max(score)));
                return;
            }
            for index in start..ingredients.len() {
                recipe.push(index);
                visit(ingredients, recipe, teaspoons, index, required, best);
                recipe.pop();
            }
        }
        let mut best = None;
        visit(
            ingredients,
            &mut Vec::new(),
            teaspoons,
            0,
            required,
            &mut best,
        );
        best
    }

    #[test]
    fn synthetic_mixture_answers() {
        assert_eq!(part1(MIXTURE).unwrap(), "400000000");
        assert_eq!(part2(MIXTURE).unwrap(), "400000000");
    }

    #[test]
    fn quantities_match_expanded_recipes() {
        let mut seed = 15u64;
        let mut cases = 0;
        for count in 1..=4 {
            for teaspoons in 0..=6 {
                for _ in 0..5 {
                    let mut ingredients = Vec::new();
                    for _ in 0..count {
                        let mut values = [0; 5];
                        for value in &mut values {
                            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                            *value = ((seed >> 32) % 9) as i64 - 3;
                        }
                        ingredients.push(Attributes {
                            capacity: values[0],
                            durability: values[1],
                            flavor: values[2],
                            texture: values[3],
                            calories: values[4],
                        });
                    }
                    let store = Store { ingredients };
                    let before = store.ingredients.clone();
                    for required in [None, Some(5)] {
                        let expected =
                            expanded_recipe_reference(&store.ingredients, teaspoons, required);
                        let actual = store.best_recipe(teaspoons, required);
                        match expected {
                            Some(score) => assert_eq!(actual.unwrap(), score),
                            None => assert!(matches!(actual, Err(SolveError::InvalidInput(_)))),
                        }
                        cases += 1;
                    }
                    assert_eq!(store.ingredients, before);
                }
            }
        }
        assert_eq!(cases, 280);
    }

    #[test]
    fn feasible_zero_and_impossible_calories_are_distinct() {
        let zero = "Bad: capacity -1, durability 0, flavor 1, texture 1, calories 5";
        assert_eq!(part1(zero).unwrap(), "0");
        assert_eq!(part2(zero).unwrap(), "0");
        let impossible = SINGLE.replace("calories 5", "calories 1");
        assert!(matches!(
            part2(&impossible),
            Err(SolveError::InvalidInput(_))
        ));
        assert_eq!(part1(SINGLE).unwrap(), "100000000");
        assert_eq!(part2(SINGLE).unwrap(), "100000000");
    }

    #[test]
    fn unused_ingredients_and_zero_teaspoons_are_supported() {
        let input =
            format!("{SINGLE}\nBad: capacity -1, durability -1, flavor -1, texture -1, calories 7");
        assert_eq!(part1(&input).unwrap(), "100000000");
        assert_eq!(part2(&input).unwrap(), "100000000");
        let store = Store::parse(SINGLE).unwrap();
        assert_eq!(store.best_recipe(0, None).unwrap(), 0);
        assert_eq!(store.best_recipe(0, Some(0)).unwrap(), 0);
        assert!(store.best_recipe(0, Some(1)).is_err());
        let identical = format!("{SINGLE}\n{}", SINGLE.replace("Sugar:", "Other:"));
        assert_eq!(part1(&identical).unwrap(), "100000000");
    }

    #[test]
    fn all_five_properties_are_named_and_unique() {
        let reordered = "Sugar: calories 5, texture 1, capacity 1, flavor 1, durability 1";
        assert_eq!(part1(reordered).unwrap(), "100000000");
        assert_eq!(part2(&MIXTURE.replace('\n', "\r\n")).unwrap(), "400000000");
        for input in [
            "Sugar: capacity 1, durability 1, flavor 1, texture 1",
            "Sugar: capacity 1, durability 1, flavor 1, texture 1, calories 5, capacity 1",
            "Sugar: capacity 1, durability 1, flavor 1, texture 1, unknown 5",
            "Sugar: capacity 1, durability 1, flavor 1, texture 1, calories 5, unknown 0",
            ": capacity 1, durability 1, flavor 1, texture 1, calories 5",
        ] {
            assert!(Store::parse(input).is_err(), "{input}");
        }
        let duplicate = format!("{SINGLE}\n{SINGLE}");
        assert!(Store::parse(&duplicate)
            .err()
            .unwrap()
            .to_string()
            .starts_with("line 2:"));
    }

    #[test]
    fn malformed_and_overflowing_numbers_keep_context() {
        use std::error::Error;
        for input in ["", " \n"] {
            assert!(matches!(Store::parse(input), Err(SolveError::EmptyInput)));
        }
        for line in [
            "invalid",
            "Sugar: capacity no, durability 1, flavor 1, texture 1, calories 5",
            "",
        ] {
            let input = format!("{SINGLE}\n{line}\n");
            for solver in [part1, part2] {
                assert!(solver(&input)
                    .unwrap_err()
                    .to_string()
                    .starts_with("line 2:"));
            }
        }
        let input = SINGLE.replace(
            "capacity 1",
            "capacity 9999999999999999999999999999999999999",
        );
        let error = part1(&input).unwrap_err();
        assert!(error.to_string().starts_with("line 1:"));
        assert!(error.source().unwrap().source().is_some());
        let (_, attrs) = parse_record(
            "Min: capacity -9223372036854775808, durability 1, flavor 1, texture 1, calories 5",
        )
        .unwrap();
        assert_eq!(attrs.capacity, i64::MIN);
    }

    #[test]
    fn scaling_addition_calories_and_score_use_checked_arithmetic() {
        let huge = Attributes {
            capacity: i64::MAX,
            durability: 2,
            flavor: 1,
            texture: 1,
            calories: 0,
        };
        assert!(matches!(huge.score(), Err(SolveError::ArithmeticOverflow)));
        assert!(matches!(
            Attributes::default().add_scaled(huge, 2),
            Err(SolveError::ArithmeticOverflow)
        ));
        assert!(matches!(
            huge.add_scaled(
                Attributes {
                    capacity: 1,
                    ..Attributes::default()
                },
                1
            ),
            Err(SolveError::ArithmeticOverflow)
        ));
        assert!(matches!(
            Attributes::default().add_scaled(
                Attributes {
                    calories: i64::MAX,
                    ..Attributes::default()
                },
                2
            ),
            Err(SolveError::ArithmeticOverflow)
        ));
        let zero = Store {
            ingredients: vec![Attributes {
                capacity: i64::MAX / 100,
                durability: i64::MAX / 100,
                flavor: 0,
                texture: 1,
                calories: 5,
            }],
        };
        assert_eq!(zero.best_recipe(100, None).unwrap(), 0);
    }

    #[test]
    fn calorie_filter_precedes_quality_product() {
        let input = format!("{SINGLE}\nHigh: capacity 1000000, durability 1000000, flavor 1000000, texture 1000000, calories 1");
        assert!(matches!(part1(&input), Err(SolveError::ArithmeticOverflow)));
        assert_eq!(part2(&input).unwrap(), "100000000");
    }

    #[test]
    fn excessive_recipe_counts_are_rejected_before_search() {
        assert!(check_recipe_budget(4, 100).is_ok());
        assert!(check_recipe_budget(5, 100).is_ok());
        assert!(check_recipe_budget(6, 100).is_err());
        assert!(check_recipe_budget(0, 100).is_err());
        assert!(check_recipe_budget(usize::MAX, usize::MAX).is_err());
        assert!(Store {
            ingredients: vec![Attributes::default(); 6]
        }
        .best_recipe(100, None)
        .is_err());
    }

    #[test]
    fn test_part1_input() {
        scaffold_test_result(2015, 15, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(2015, 15, "input.txt", "input-spec.2.txt", part2);
    }
}
