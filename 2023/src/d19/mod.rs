extern crate core;

use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;

const ACCEPTED: WorkflowName = WorkflowName('A', ' ', ' ');

#[derive(Debug, PartialEq)]
enum Rating {
    // eXtremely Cool Looking
    X, 
    // Musical
    M,
    // Aerodynamic
    A,
    // Shiny
    S,
}

#[derive(Debug, PartialEq)]
struct Part {
    x: i64,
    m: i64,
    a: i64,
    s: i64,
}

impl Part {
    fn get(&self, rating: &Rating) -> i64 {
        match rating {
            Rating::X => self.x,
            Rating::M => self.m,
            Rating::A => self.a,
            Rating::S => self.s,
        }
    }

    fn matches(&self, workstep: &WorkflowStep) -> bool {
        self.get(&workstep.rating).cmp(&workstep.comparator) == workstep.comparison
    }

    fn total(&self) -> i64 {
        self.x + self.m + self.a + self.s
    }
}

impl FromStr for Part {
    type Err = ParseInputError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let Some(line) = line.strip_prefix('{') else {
            return Err(ParseInputError);
        };
        let Some(line) = line.strip_suffix('}') else {
            return Err(ParseInputError);
        };

        let mut x: Result<i64, ParseInputError> = Err(ParseInputError);
        let mut m: Result<i64, ParseInputError> = Err(ParseInputError);
        let mut a: Result<i64, ParseInputError> = Err(ParseInputError);
        let mut s: Result<i64, ParseInputError> = Err(ParseInputError);

        for element in line.split(',') {
            let Some((var, value_str)) = element.split_once('=') else {
                return Err(ParseInputError);
            };
            let value: i64 = value_str.parse().map_err(|_| ParseInputError)?;
            match var {
                "x" => x = Ok(value),
                "m" => m = Ok(value),
                "a" => a = Ok(value),
                "s" => s = Ok(value),
                _ => return Err(ParseInputError),
            }
        }

        let x = x?;
        let m = m?;
        let a = a?;
        let s = s?;
        Ok(Self { x, m, a, s })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct WorkflowName(char, char, char);
// FIXME: Would be great to improve but was fighting the compiler.

impl FromStr for WorkflowName {
    type Err = ParseInputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        Ok(WorkflowName(
            chars.next().unwrap_or(' '),
            chars.next().unwrap_or(' '),
            chars.next().unwrap_or(' '),
        ))
    }
}

#[derive(Debug, PartialEq)]
struct WorkflowStep {
    rating: Rating,
    comparison: Ordering,
    comparator: i64,
    target: WorkflowName,
}

impl FromStr for WorkflowStep {
    type Err = ParseInputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((s, target_str)) = s.split_once(':') else {
            return Err(ParseInputError);
        };

        let mut chars = s.chars();
        let rating = match chars.next() {
            Some('x') => Rating::X,
            Some('m') => Rating::M,
            Some('a') => Rating::A,
            Some('s') => Rating::S,
            _ => return Err(ParseInputError),
        };
        let comparison = match chars.next() {
            Some('>') => Ordering::Greater,
            Some('<') => Ordering::Less,
            _ => return Err(ParseInputError),
        };
        let comparator = s[2..].parse().map_err(|_| ParseInputError)?;
        let target = target_str.parse()?;

        Ok(Self {
            rating,
            comparison,
            comparator,
            target,
        })
    }
}

#[derive(Debug, PartialEq)]
struct Workflow {
    name: WorkflowName,
    steps: Vec<WorkflowStep>,
    default: WorkflowName,
}

impl Workflow {
    fn process(&self, part: &Part) -> WorkflowName {
        for step in &self.steps {
            if part.matches(step) {
                return step.target;
            }
        }
        self.default
    }

    fn process_tesseract(&self, tesseract: StateSpace, queue: &mut VecDeque<PossibilityState>) {
        let mut next_tesseract = Some(tesseract);

        for step in &self.steps {
            let Some(consider) = next_tesseract else {
                break;
            };

            let (split, retain) = consider.split(step);

            if let Some(split) = split {
                queue.push_back(split);
            }

            next_tesseract = retain;
        }

        if let Some(default_tesseract) = next_tesseract {
            queue.push_back(PossibilityState {
                state_space: default_tesseract,
                workflow_name: self.default,
            });
        }
    }
}

impl FromStr for Workflow {
    type Err = ParseInputError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let Some(line) = line.strip_suffix('}') else {
            return Err(ParseInputError);
        };
        let Some((name_str, steps_str)) = line.split_once('{') else {
            return Err(ParseInputError);
        };

        let name = name_str.parse()?;
        let mut steps = Vec::new();
        let mut default: Result<WorkflowName, ParseInputError> = Err(ParseInputError);

        for step in steps_str.split(',') {
            if let Ok(step) = WorkflowStep::from_str(step) {
                steps.push(step);
            } else {
                default = WorkflowName::from_str(step);
            }
        }

        let default = default?;

        Ok(Self {
            name,
            steps,
            default,
        })
    }
}

// FIXME: THIS WOULD HAVE BEEN SWEET FOR DAY 5. Let's try to retrofit later.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Range(i64, i64);

impl Range {
    fn range(self) -> i64 {
        1 + self.1 - self.0
    }

    fn split(self, comparison: Ordering, comparator: i64) -> (Option<Self>, Option<Self>) {
        let mut split: Option<Self> = None;
        let mut retain: Option<Self> = None;

        match comparison {
            Ordering::Greater => {
                if comparator >= self.1 {
                    retain = Some(self);
                } else if comparator < self.0 {
                    split = Some(self);
                } else {
                    retain = Some(Self(self.0, comparator));
                    split = Some(Self(comparator + 1, self.1));
                }
            }
            Ordering::Less => {
                if comparator > self.1 {
                    split = Some(self);
                } else if comparator <= self.0 {
                    retain = Some(self);
                } else {
                    retain = Some(Self(comparator, self.1));
                    split = Some(Self(self.0, comparator.saturating_sub(1)));
                }
            }
            Ordering::Equal => (),
        }

        (split, retain)
    }
}

#[derive(Debug, PartialEq)]
struct PossibilityState {
    state_space: StateSpace,
    workflow_name: WorkflowName,
}

#[derive(Debug, PartialEq)]
struct StateSpace {
    x: Range,
    m: Range,
    a: Range,
    s: Range,
}

impl StateSpace {
    fn initial() -> Self {
        Self {
            x: Range(1, 4000),
            m: Range(1, 4000),
            a: Range(1, 4000),
            s: Range(1, 4000),
        }
    }

    fn get(&self, rating: &Rating) -> Range {
        match rating {
            Rating::X => self.x,
            Rating::M => self.m,
            Rating::A => self.a,
            Rating::S => self.s,
        }
    }

    fn split_space(&self, rating: &Rating, range: Range) -> Self {
        match rating {
            Rating::X => Self { x: range, ..*self },
            Rating::M => Self { m: range, ..*self },
            Rating::A => Self { a: range, ..*self },
            Rating::S => Self { s: range, ..*self },
        }
    }

    fn split(&self, step: &WorkflowStep) -> (Option<PossibilityState>, Option<StateSpace>) {
        let range = self.get(&step.rating);
        let (split, retain) = range.split(step.comparison, step.comparator);

        let split = split.map(|rg| PossibilityState {
            state_space: self.split_space(&step.rating, rg),
            workflow_name: step.target,
        });
        let retain = retain.map(|rg| self.split_space(&step.rating, rg));

        (split, retain)
    }

    fn volume(&self) -> i64 {
        let x = i64::from(self.x.range());
        let m = i64::from(self.m.range());
        let a = i64::from(self.a.range());
        let s = i64::from(self.s.range());
        x * m * a * s
    }
}

#[derive(Debug)]
struct WorkflowSystem {
    workflows: HashMap<WorkflowName, Workflow>,
    parts: Vec<Part>,
}

impl WorkflowSystem {
    fn process(&self, part: &Part) -> WorkflowName {
        let mut location = WorkflowName('i', 'n', ' ');

        while let Some(workflow) = self.workflows.get(&location) {
            location = workflow.process(part);
        }

        location
    }

    fn total_of_accepted_parts(&self) -> i64 {
        self.parts
            .iter()
            .filter_map(|part| {
                if self.process(part) == ACCEPTED {
                    Some(part.total())
                } else {
                    None
                }
            })
            .sum()
    }

    fn accepted_possibilities(&self) -> i64 {
        let mut total = 0;
        let mut queue = VecDeque::new();
        queue.push_back(PossibilityState {
            state_space: StateSpace::initial(),
            workflow_name: WorkflowName('i', 'n', ' '),
        });

        while let Some(state) = queue.pop_front() {
            if state.workflow_name == ACCEPTED {
                total += state.state_space.volume();
                continue;
            }

            if let Some(workflow) = self.workflows.get(&state.workflow_name) {
                workflow.process_tesseract(state.state_space, &mut queue);
            }
        }

        total
    }
}

impl FromStr for WorkflowSystem {
    type Err = ParseInputError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let Some((workflows_str, parts_str)) = input.split_once("\n\n") else {
            return Err(ParseInputError);
        };

        let mut workflows = HashMap::new();
        for workflow in workflows_str.lines() {
            let workflow: Workflow = workflow.parse()?;
            workflows.insert(workflow.name, workflow);
        }

        let mut parts = Vec::new();
        for part in parts_str.lines() {
            let part = part.parse()?;
            parts.push(part);
        }

        Ok(Self { workflows, parts })
    }
}

#[derive(Debug, PartialEq)]
struct ParseInputError;

pub fn fn1(input: &str) -> i64 {
    let system = WorkflowSystem::from_str(input).unwrap();
    system.total_of_accepted_parts() as i64
}

pub fn fn2(input: &str) -> i64 {
    let system = WorkflowSystem::from_str(input).unwrap();
    system.accepted_possibilities() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test;

    const YEAR: i16 = 2023;
    const DAY: i16 = 19;

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
