use num_rational::BigRational;
use num_traits::Zero;
use svutils::SolveError;

const UNKNOWNS: usize = 6;
type Row = [BigRational; UNKNOWNS + 1];

pub(super) struct LinearSystem {
    pivots: [Option<Row>; UNKNOWNS],
}

impl LinearSystem {
    pub(super) fn new() -> Self {
        Self {
            pivots: std::array::from_fn(|_| None),
        }
    }

    pub(super) fn add_row(&mut self, mut row: Row) -> Result<(), SolveError> {
        for column in 0..UNKNOWNS {
            if row[column].is_zero() {
                continue;
            }
            if let Some(pivot) = &self.pivots[column] {
                let factor = row[column].clone();
                for j in column..=UNKNOWNS {
                    row[j] -= &factor * &pivot[j];
                }
            } else {
                let divisor = row[column].clone();
                for value in &mut row[column..] {
                    *value /= &divisor;
                }
                self.pivots[column] = Some(row);
                return Ok(());
            }
        }
        if row[UNKNOWNS].is_zero() {
            Ok(())
        } else {
            Err(SolveError::InvalidInput(
                "hailstone equations are inconsistent".into(),
            ))
        }
    }

    pub(super) fn is_complete(&self) -> bool {
        self.pivots.iter().all(Option::is_some)
    }

    pub(super) fn solution(&self) -> Result<[BigRational; UNKNOWNS], SolveError> {
        let mut result = std::array::from_fn(|_| BigRational::zero());
        for column in (0..UNKNOWNS).rev() {
            let row = self.pivots[column].as_ref().ok_or_else(|| {
                SolveError::InvalidInput(
                    "hailstone equations do not determine a unique rock".into(),
                )
            })?;
            let mut value = row[UNKNOWNS].clone();
            for j in column + 1..UNKNOWNS {
                value -= &row[j] * &result[j];
            }
            result[column] = value;
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rational(n: i64, d: i64) -> BigRational {
        BigRational::new(n.into(), d.into())
    }

    #[test]
    fn solves_exact_fractions_with_reordered_and_redundant_rows() {
        let expected = [
            rational(1, 2),
            rational(-2, 3),
            rational(5, 7),
            rational(11, 1),
            rational(-13, 1),
            rational(17, 1),
        ];
        let mut system = LinearSystem::new();
        for i in (0..UNKNOWNS).rev() {
            let mut row = std::array::from_fn(|_| BigRational::zero());
            row[i] = rational(2 * (i as i64 + 1), 1);
            row[UNKNOWNS] = &row[i] * &expected[i];
            if i + 1 < UNKNOWNS {
                row[i + 1] = rational(1, 1);
                row[UNKNOWNS] += &expected[i + 1];
            }
            system.add_row(row.clone()).unwrap();
            system.add_row(row).unwrap();
        }
        system
            .add_row(std::array::from_fn(|_| BigRational::zero()))
            .unwrap();
        assert_eq!(system.solution().unwrap(), expected);
    }

    #[test]
    fn eliminates_coupled_equations() {
        let mut system = LinearSystem::new();
        for values in [
            [1, 1, 0, 0, 0, 0, 5],
            [1, -1, 0, 0, 0, 0, 1],
            [0, 0, 1, 0, 0, 0, 0],
            [0, 0, 0, 1, 0, 0, 0],
            [0, 0, 0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0, 1, 0],
        ] {
            system.add_row(values.map(|v| rational(v, 1))).unwrap();
        }
        assert_eq!(
            system.solution().unwrap(),
            [3, 2, 0, 0, 0, 0].map(|v| rational(v, 1))
        );
    }

    #[test]
    fn distinguishes_insufficient_information_from_contradiction() {
        let mut system = LinearSystem::new();
        assert!(system
            .solution()
            .unwrap_err()
            .to_string()
            .contains("unique rock"));
        let mut row = std::array::from_fn(|_| BigRational::zero());
        row[0] = rational(1, 1);
        row[UNKNOWNS] = rational(1, 3);
        system.add_row(row.clone()).unwrap();
        assert!(system.solution().is_err());
        row[UNKNOWNS] += rational(1, 1_000_000_000);
        assert!(system
            .add_row(row)
            .unwrap_err()
            .to_string()
            .contains("inconsistent"));
    }
}
