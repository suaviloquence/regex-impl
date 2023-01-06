#[derive(Debug, Clone, PartialEq, Copy)]
enum MatchValue {
    Char(char),
    /// `branch` must always be valid
    Split {
        branch: usize,
    },
    Wildcard,
    Match,
}

impl<'a> MatchValue {
    /// Assumes MatchValue is either `Char` or `Wildcard`
    fn matches(&self, value: char) -> bool {
        match self {
            MatchValue::Char(c) => *c == value,
            MatchValue::Wildcard => true,
            _ => unreachable!("called MatchValue::matches() on MatchValue::Split"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct State {
    value: MatchValue,
    next: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Regex {
    states: Vec<State>,
    head: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    MissingValue,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingValue => f.write_str("Missing value for modifier"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Default)]
struct Step<'a> {
    states: &'a [State],
    // TODO: switch to bit field for space efficiency - Vec<u8>
    current: Vec<bool>,
    next: Vec<bool>,
    matched: bool,
}

impl<'a> Step<'a> {
    fn new(states: &'a [State]) -> Self {
        Self {
            states,
            current: vec![false; states.len()],
            next: vec![false; states.len()],
            matched: false,
        }
    }

    fn add_state(&mut self, idx: usize) {
        let state = &self.states[idx];

        match state.value {
            MatchValue::Char(_) | MatchValue::Wildcard => {
                self.next[idx] = true;
            }
            MatchValue::Split { branch } => {
                self.add_state(branch);
                self.add_state(state.next);
            }
            MatchValue::Match => self.matched = true,
        };
    }

    fn step(&mut self, to_match: char) {
        std::mem::swap(&mut self.current, &mut self.next);

        // TODO: better way to do this
        for v in &mut self.next {
            *v = false;
        }

        let next_states: Vec<_> = self
            .current
            .iter()
            .enumerate()
            .filter(|(_, x)| **x)
            .map(|(i, _)| &self.states[i])
            .filter(|state| state.value.matches(to_match))
            .map(|s| s.next)
            .collect();

        for next in next_states {
            self.add_state(next);
        }
    }
}

#[inline]
fn get_value(c: Option<char>) -> Result<MatchValue, Error> {
    match c {
        Some('?' | '*' | '+') | None => Err(Error::MissingValue),
        Some('.') => Ok(MatchValue::Wildcard),
        Some(c) => Ok(MatchValue::Char(c)),
    }
}

impl Regex {
    pub fn from_simple_expression(expression: &str) -> Result<Self, Error> {
        let mut states = vec![State {
            next: 0,
            value: MatchValue::Match,
        }];

        let mut chars = expression.chars().rev();

        let mut index = 0;

        macro_rules! push {
            ($state: expr) => {
                index += 1;
                states.push($state);
            };
        }

        let mut token = Vec::new();

        while let Some(c) = chars.next() {
            match c {
                '?' => {
                    let value = get_value(chars.next())?;

                    // next, matcher, split
                    // idx, idx + 1, idx + 2
                    let matcher = State { value, next: index };
                    let split = State {
                        value: MatchValue::Split { branch: index + 1 },
                        next: index,
                    };

                    push!(matcher);
                    push!(split);
                }
                '+' => {
                    let value = get_value(chars.next())?;

                    // next, split, matcher
                    // idx,  idx+1, idx + 2

                    let split = State {
                        value: MatchValue::Split { branch: index + 2 },
                        next: index,
                    };

                    let matcher = State {
                        value,
                        next: index + 1,
                    };

                    push!(split);
                    push!(matcher);
                }
                '*' => {
                    let value = get_value(chars.next())?;

                    // next, matcher, split
                    // idx, idx + 1, idx + 2

                    let matcher = State {
                        value,
                        next: index + 2,
                    };

                    let split = State {
                        value: MatchValue::Split { branch: index + 1 },
                        next: index,
                    };

                    push!(matcher);
                    push!(split);
                }
                '|' => {
                    // a|b
                    // b, a    , split
                    // i, i + 1, i + 2
                    if index == 0 {
                        return Err(Error::MissingValue);
                    }

                    let b = &states[index];

                    let a = State {
                        value: get_value(chars.next())?,
                        next: b.next,
                    };

                    // we add `branch` first (before `next`), so since `a` should come before `b`, `branch` is `a`
                    let split = State {
                        value: MatchValue::Split { branch: index + 1 },
                        next: index,
                    };

                    push!(a);
                    push!(split);
                }
                '/' => {
                    let c = chars.next().ok_or(Error::MissingValue)?;

                    let state = State {
                        value: MatchValue::Char(c),
                        next: index,
                    };

                    push!(state);
                }
                '.' => {
                    token.push(MatchValue::Wildcard);

                    let state = State {
                        value: MatchValue::Wildcard,
                        next: index,
                    };

                    push!(state);
                }
                _ => {
                    let state = State {
                        value: MatchValue::Char(c),
                        next: index,
                    };

                    push!(state);
                }
            }
        }

        Ok(Self {
            head: index,
            states,
        })
    }

    pub fn test(&self, string: &str) -> bool {
        let mut step = Step::new(&self.states);

        for ch in string.chars() {
            step.add_state(self.head);
            step.step(ch);
        }

        step.matched
    }
}
