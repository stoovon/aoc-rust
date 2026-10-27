use std::collections::{HashMap, HashSet, VecDeque};
use std::str::FromStr;
use svutils::SolveError;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Comparison {
    LessThan,
    GreaterThan,
}

impl Comparison {
    fn matches(self, value: i64, threshold: i64) -> bool {
        match self {
            Self::LessThan => value < threshold,
            Self::GreaterThan => value > threshold,
        }
    }
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
        workstep
            .comparison
            .matches(self.get(&workstep.rating), workstep.comparator)
    }

    fn total(&self) -> Result<i64, SolveError> {
        self.x
            .checked_add(self.m)
            .and_then(|sum| sum.checked_add(self.a))
            .and_then(|sum| sum.checked_add(self.s))
            .ok_or(SolveError::ArithmeticOverflow)
    }
}

fn decimal(text: &str) -> Result<i64, SolveError> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SolveError::InvalidInput(
            "expected a nonnegative decimal integer".into(),
        ));
    }
    Ok(text.parse()?)
}

impl FromStr for Part {
    type Err = SolveError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let body = line
            .strip_prefix('{')
            .and_then(|line| line.strip_suffix('}'))
            .ok_or_else(|| {
                SolveError::InvalidInput("expected part in '{x=...,m=...,a=...,s=...}' form".into())
            })?;
        let (mut x, mut m, mut a, mut s) = (None, None, None, None);
        for field in body.split(',') {
            let (name, value) = field
                .split_once('=')
                .ok_or_else(|| SolveError::InvalidInput("expected rating=value".into()))?;
            let slot = match name {
                "x" => &mut x,
                "m" => &mut m,
                "a" => &mut a,
                "s" => &mut s,
                _ => return Err(SolveError::InvalidInput(format!("unknown rating '{name}'"))),
            };
            if slot.replace(decimal(value)?).is_some() {
                return Err(SolveError::InvalidInput(format!(
                    "duplicate rating '{name}'"
                )));
            }
        }
        let required = |value: Option<i64>, name| {
            value.ok_or_else(|| SolveError::InvalidInput(format!("missing rating '{name}'")))
        };
        Ok(Self {
            x: required(x, "x")?,
            m: required(m, "m")?,
            a: required(a, "a")?,
            s: required(s, "s")?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Destination<'a> {
    Accept,
    Reject,
    Workflow(&'a str),
}

fn workflow_name(name: &str) -> Result<&str, SolveError> {
    if name.is_empty() || !name.bytes().all(|byte| byte.is_ascii_lowercase()) {
        return Err(SolveError::InvalidInput(
            "workflow name must contain lowercase ASCII letters".into(),
        ));
    }
    Ok(name)
}

impl<'a> Destination<'a> {
    fn parse(text: &'a str) -> Result<Self, SolveError> {
        match text {
            "A" => Ok(Self::Accept),
            "R" => Ok(Self::Reject),
            name => Ok(Self::Workflow(workflow_name(name)?)),
        }
    }
}

#[derive(Debug, PartialEq)]
struct WorkflowStep<'a> {
    rating: Rating,
    comparison: Comparison,
    comparator: i64,
    target: Destination<'a>,
}

impl<'a> WorkflowStep<'a> {
    fn parse(s: &'a str) -> Result<Self, SolveError> {
        let Some((s, target_str)) = s.split_once(':') else {
            return Err(SolveError::InvalidInput(
                "expected condition:destination".into(),
            ));
        };

        let mut chars = s.chars();
        let rating = match chars.next() {
            Some('x') => Rating::X,
            Some('m') => Rating::M,
            Some('a') => Rating::A,
            Some('s') => Rating::S,
            _ => {
                return Err(SolveError::InvalidInput(
                    "condition rating must be x, m, a or s".into(),
                ))
            }
        };
        let comparison = match chars.next() {
            Some('>') => Comparison::GreaterThan,
            Some('<') => Comparison::LessThan,
            _ => {
                return Err(SolveError::InvalidInput(
                    "condition operator must be < or >".into(),
                ))
            }
        };
        let comparator = decimal(chars.as_str())?;
        let target = Destination::parse(target_str)?;

        Ok(Self {
            rating,
            comparison,
            comparator,
            target,
        })
    }
}

#[derive(Debug, PartialEq)]
struct Workflow<'a> {
    name: &'a str,
    steps: Vec<WorkflowStep<'a>>,
    default: Destination<'a>,
}

impl<'a> Workflow<'a> {
    fn process(&self, part: &Part) -> Destination<'a> {
        for step in &self.steps {
            if part.matches(step) {
                return step.target;
            }
        }
        self.default
    }

    fn process_tesseract(&self, tesseract: StateSpace, queue: &mut VecDeque<PossibilityState<'a>>) {
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
                destination: self.default,
            });
        }
    }
}

impl<'a> Workflow<'a> {
    fn parse(line: &'a str) -> Result<Self, SolveError> {
        let body = line
            .strip_suffix('}')
            .ok_or_else(|| SolveError::InvalidInput("workflow must end with '}'".into()))?;
        let (name, rules) = body
            .split_once('{')
            .ok_or_else(|| SolveError::InvalidInput("workflow must contain '{'".into()))?;
        let name = workflow_name(name)?;
        let mut fields = rules.split(',');
        let default = Destination::parse(fields.next_back().ok_or_else(|| {
            SolveError::InvalidInput("workflow requires a default destination".into())
        })?)?;
        let steps = fields
            .map(WorkflowStep::parse)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            name,
            steps,
            default,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Range(i64, i64);

impl Range {
    fn len(self) -> i64 {
        1 + self.1 - self.0
    }

    // Return the matching interval and the remainder, each possibly empty.
    fn split(self, comparison: Comparison, comparator: i64) -> (Option<Self>, Option<Self>) {
        match comparison {
            Comparison::GreaterThan => {
                if comparator >= self.1 {
                    (None, Some(self))
                } else if comparator < self.0 {
                    (Some(self), None)
                } else {
                    // comparator < upper guarantees comparator + 1 cannot overflow.
                    (
                        Some(Self(comparator + 1, self.1)),
                        Some(Self(self.0, comparator)),
                    )
                }
            }
            Comparison::LessThan => {
                if comparator > self.1 {
                    (Some(self), None)
                } else if comparator <= self.0 {
                    (None, Some(self))
                } else {
                    // comparator > lower guarantees comparator - 1 cannot overflow.
                    (
                        Some(Self(self.0, comparator - 1)),
                        Some(Self(comparator, self.1)),
                    )
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
struct PossibilityState<'a> {
    state_space: StateSpace,
    destination: Destination<'a>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

    fn split<'a>(
        &self,
        step: &WorkflowStep<'a>,
    ) -> (Option<PossibilityState<'a>>, Option<StateSpace>) {
        let range = self.get(&step.rating);
        let (split, retain) = range.split(step.comparison, step.comparator);

        let split = split.map(|rg| PossibilityState {
            state_space: self.split_space(&step.rating, rg),
            destination: step.target,
        });
        let retain = retain.map(|rg| self.split_space(&step.rating, rg));

        (split, retain)
    }

    // Every axis is a nonempty subset of 1..=4000; the product fits i64.
    fn volume(&self) -> i64 {
        let x = self.x.len();
        let m = self.m.len();
        let a = self.a.len();
        let s = self.s.len();
        x * m * a * s
    }
}

#[derive(Debug)]
struct WorkflowSystem<'a> {
    workflows: HashMap<&'a str, Workflow<'a>>,
    parts: Vec<Part>,
}

impl<'a> WorkflowSystem<'a> {
    fn process(&self, part: &Part) -> Result<Destination<'a>, SolveError> {
        let mut location = Destination::Workflow("in");
        let mut visited = HashSet::new();
        while let Destination::Workflow(name) = location {
            if !visited.insert(name) {
                return Err(SolveError::InvalidInput(format!(
                    "part loops at workflow '{name}'"
                )));
            }
            location = self.workflows[name].process(part);
        }
        Ok(location)
    }

    fn total_of_accepted_parts(&self) -> Result<i64, SolveError> {
        self.parts.iter().try_fold(0_i64, |total, part| {
            if self.process(part)? == Destination::Accept {
                total
                    .checked_add(part.total()?)
                    .ok_or(SolveError::ArithmeticOverflow)
            } else {
                Ok(total)
            }
        })
    }

    fn accepted_possibilities(&self) -> Result<i64, SolveError> {
        self.count_accepted(StateSpace::initial())
    }

    fn count_accepted(&self, initial: StateSpace) -> Result<i64, SolveError> {
        let mut total = 0_i64;
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_back(PossibilityState {
            state_space: initial,
            destination: Destination::Workflow("in"),
        });
        while let Some(state) = queue.pop_front() {
            match state.destination {
                Destination::Accept => {
                    total = total
                        .checked_add(state.state_space.volume())
                        .ok_or(SolveError::ArithmeticOverflow)?;
                }
                Destination::Reject => {}
                Destination::Workflow(name) => {
                    // Splits partition the input domain: repeating an identical box
                    // at a workflow means those same parts are circulating forever.
                    if !visited.insert((name, state.state_space)) {
                        return Err(SolveError::InvalidInput(format!(
                            "rating range loops at workflow '{name}'"
                        )));
                    }
                    self.workflows[name].process_tesseract(state.state_space, &mut queue);
                }
            }
        }
        Ok(total)
    }
}

impl<'a> WorkflowSystem<'a> {
    fn parse(input: &'a str) -> Result<Self, SolveError> {
        if input.trim().is_empty() {
            return Err(SolveError::EmptyInput);
        }
        let mut lines = input.lines().enumerate();
        let mut workflows = HashMap::new();
        let mut workflow_lines = HashMap::new();
        let mut separator = false;
        for (index, line) in lines.by_ref() {
            if line.is_empty() {
                separator = true;
                break;
            }
            let workflow = Workflow::parse(line).map_err(|error| error.at_line(index + 1))?;
            let name = workflow.name;
            if workflows.insert(name, workflow).is_some() {
                return Err(
                    SolveError::InvalidInput(format!("duplicate workflow '{name}'"))
                        .at_line(index + 1),
                );
            }
            workflow_lines.insert(name, index + 1);
        }
        if !separator {
            return Err(SolveError::InvalidInput(
                "expected blank line between workflows and parts".into(),
            ));
        }
        if !workflows.contains_key("in") {
            return Err(SolveError::InvalidInput(
                "missing entry workflow 'in'".into(),
            ));
        }
        for workflow in workflows.values() {
            for target in workflow
                .steps
                .iter()
                .map(|step| step.target)
                .chain(std::iter::once(workflow.default))
            {
                if let Destination::Workflow(name) = target {
                    if !workflows.contains_key(name) {
                        return Err(SolveError::InvalidInput(format!(
                            "undefined destination '{name}'"
                        ))
                        .at_line(workflow_lines[workflow.name]));
                    }
                }
            }
        }
        let parts = lines
            .map(|(index, line)| {
                line.parse::<Part>()
                    .map_err(|error| error.at_line(index + 1))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { workflows, parts })
    }
}

pub fn part1(input: &str) -> Result<String, SolveError> {
    let system = WorkflowSystem::parse(input)?;
    Ok(system.total_of_accepted_parts()?.to_string())
}

pub fn part2(input: &str) -> Result<String, SolveError> {
    let system = WorkflowSystem::parse(input)?;
    Ok(system.accepted_possibilities()?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use svutils::scaffold_test_result;

    const YEAR: i16 = 2023;
    const DAY: i16 = 19;

    #[test]
    fn test_part1_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.1.txt", part1);
    }

    #[test]
    fn test_part2_input() {
        scaffold_test_result(YEAR, DAY, "input.txt", "input-spec.2.txt", part2);
    }
    #[test]
    fn full_names_and_explicit_destinations() {
        let input = "in{x<5:abcfirst,abcsecond}\nabcfirst{A}\nabcsecond{R}\n\n{x=1,m=2,a=3,s=4}\n{x=5,m=2,a=3,s=4}";
        let system = WorkflowSystem::parse(input).unwrap();
        assert_eq!(system.total_of_accepted_parts().unwrap(), 10);
        assert_eq!(
            system.accepted_possibilities().unwrap(),
            4 * 4000_i64.pow(3)
        );
    }

    #[test]
    fn workflow_structure_and_references_are_validated() {
        for workflows in [
            "in{missing}",
            "other{A}",
            "in{A}\nin{R}",
            "in{A,x<3:R}",
            "in{x=3:A,R}",
            "in{x<3:A}",
            "in{,A}",
            "in{A}\nBAD{R}",
            "in{A}\nunused{missing}",
        ] {
            let input = format!("{workflows}\n\n{{x=1,m=2,a=3,s=4}}");
            assert!(WorkflowSystem::parse(&input).is_err(), "{workflows}");
        }
    }
    #[test]
    fn encountered_cycles_are_errors() {
        for workflows in [
            "in{in}",
            "in{next}\nnext{in}",
            "in{x<3:next,A}\nnext{x<2:A,in}",
        ] {
            let input = format!("{workflows}\n\n{{x=2,m=1,a=1,s=1}}");
            assert!(matches!(part1(&input), Err(SolveError::InvalidInput(_))));
            assert!(matches!(part2(&input), Err(SolveError::InvalidInput(_))));
        }
    }

    #[test]
    fn unreachable_cycles_and_disjoint_merges_are_legal() {
        let input =
            "in{x<1:loop,x<3:merge,other}\nloop{loop}\nother{merge}\nmerge{A}\n\n{x=2,m=1,a=1,s=1}";
        assert_eq!(part1(input).unwrap(), "5");
        assert_eq!(part2(input).unwrap(), 4000_i64.pow(4).to_string());
        // A graph cycle can be harmless for listed parts but reachable in part2.
        let input = "in{x<3:in,A}\n\n{x=3,m=1,a=1,s=1}";
        assert_eq!(part1(input).unwrap(), "6");
        assert!(part2(input).is_err());
    }
    #[test]
    fn range_splits_partition_values_at_every_threshold() {
        for lower in 1..=5 {
            for upper in lower..=5 {
                let range = Range(lower, upper);
                for threshold in [i64::MIN, -1, 0, 1, 2, 3, 4, 5, 6, i64::MAX] {
                    for comparison in [Comparison::LessThan, Comparison::GreaterThan] {
                        let (matching, remainder) = range.split(comparison, threshold);
                        let values = |interval: Option<Range>| {
                            interval
                                .map(|Range(lo, hi)| {
                                    assert!(lower <= lo && lo <= hi && hi <= upper);
                                    (lo..=hi).collect::<Vec<_>>()
                                })
                                .unwrap_or_default()
                        };
                        let matching = values(matching);
                        let remainder = values(remainder);
                        for value in lower..=upper {
                            let expected = match comparison {
                                Comparison::LessThan => value < threshold,
                                Comparison::GreaterThan => value > threshold,
                            };
                            assert_eq!(comparison.matches(value, threshold), expected);
                            assert_eq!(matching.contains(&value), expected);
                            assert_eq!(remainder.contains(&value), !expected);
                        }
                    }
                }
            }
        }
        assert_eq!(
            Range(i64::MIN, i64::MIN + 1).split(Comparison::LessThan, i64::MIN + 1),
            (
                Some(Range(i64::MIN, i64::MIN)),
                Some(Range(i64::MIN + 1, i64::MIN + 1))
            )
        );
        assert_eq!(
            Range(i64::MAX - 1, i64::MAX).split(Comparison::GreaterThan, i64::MAX - 1),
            (
                Some(Range(i64::MAX, i64::MAX)),
                Some(Range(i64::MAX - 1, i64::MAX - 1))
            )
        );
    }
    #[test]
    fn parser_rejects_bad_fields_with_line_context() {
        for part in [
            "{x=1,m=2,a=3}",
            "{x=1,m=2,a=3,s=4,x=5}",
            "{x=1,m=2,a=3,z=4}",
            "{x=-1,m=2,a=3,s=4}",
            "{x=+1,m=2,a=3,s=4}",
            "{x=,m=2,a=3,s=4}",
            "{x=1,m=2,a=3,s=4,}",
            "{x=１,m=2,a=3,s=4}",
            "",
        ] {
            let input = format!("in{{A}}\n\n{part}\n");
            assert!(
                matches!(
                    WorkflowSystem::parse(&input),
                    Err(SolveError::AtLine { line: 3, .. })
                ),
                "{part}"
            );
        }
        let input = "in{A}\n\n{x=9223372036854775808,m=2,a=3,s=4}";
        assert!(
            matches!(WorkflowSystem::parse(input), Err(SolveError::AtLine {line:3,error}) if matches!(*error,SolveError::InvalidNumber(_)))
        );
        for condition in ["x<-1", "x<+1", "x<", "x<１", "é<1"] {
            let input = format!("in{{{condition}:A,R}}\n\n");
            assert!(matches!(
                WorkflowSystem::parse(&input),
                Err(SolveError::AtLine { line: 1, .. })
            ));
        }
        assert!(matches!(
            WorkflowSystem::parse("in{missing}\n\n"),
            Err(SolveError::AtLine { line: 1, .. })
        ));
        for input in ["", " \t\r\n"] {
            assert!(matches!(
                WorkflowSystem::parse(input),
                Err(SolveError::EmptyInput)
            ));
        }
    }

    #[test]
    fn parser_preserves_borrowing_and_supports_crlf_and_field_order() {
        let input = "in{x<0:R,A}\n\n{s=4,a=3,m=2,x=1}\n";
        assert_eq!(part1(input).unwrap(), "10");
        let crlf = input.replace('\n', "\r\n");
        assert_eq!(part1(&crlf).unwrap(), "10");
        assert_eq!(part2(&crlf).unwrap(), 4000_i64.pow(4).to_string());
        assert_eq!(part1("in{A}\n\n").unwrap(), "0");
        assert!(part1("in{A}\n{x=1,m=2,a=3,s=4}").is_err());
    }
    #[test]
    fn range_counting_matches_small_domain_enumeration() {
        for threshold in 0..=6 {
            for other in 0..=6 {
                let input = format!("in{{x<{threshold}:left,right}}\nleft{{m>{other}:A,next}}\nright{{a<{threshold}:next,R}}\nnext{{s>{other}:A,R}}\n\n");
                let system = WorkflowSystem::parse(&input).unwrap();
                let mut expected = 0;
                for x in 1..=4 {
                    for m in 1..=4 {
                        for a in 1..=4 {
                            for s in 1..=4 {
                                let accepted = if x < threshold {
                                    m > other || s > other
                                } else {
                                    a < threshold && s > other
                                };
                                expected += i64::from(accepted);
                                let part = Part { x, m, a, s };
                                assert_eq!(
                                    system.process(&part).unwrap() == Destination::Accept,
                                    accepted
                                );
                            }
                        }
                    }
                }
                let small = StateSpace {
                    x: Range(1, 4),
                    m: Range(1, 4),
                    a: Range(1, 4),
                    s: Range(1, 4),
                };
                assert_eq!(system.count_accepted(small).unwrap(), expected);
            }
        }
    }
}
