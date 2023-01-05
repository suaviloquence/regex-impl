use std::{
    collections::HashSet,
    rc::{Rc, Weak},
};

#[derive(Debug, Clone)]
enum Ptr<T> {
    Strong(Rc<T>),
    Weak(Weak<T>),
}

impl<T> Ptr<T> {
    /// Weak must be valid!
    unsafe fn as_ref(&self) -> &T {
        match self {
            Self::Strong(rc) => &*rc,
            Self::Weak(weak) => unsafe { &*weak.as_ptr() },
        }
    }
}

impl<T: PartialEq> PartialEq for Ptr<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Ptr::Strong(a), Ptr::Strong(b)) => a == b,
            (Ptr::Strong(s), Ptr::Weak(w)) | (Ptr::Weak(w), Ptr::Strong(s)) => match w.upgrade() {
                Some(x) => &x == s,
                None => false,
            },
            (Ptr::Weak(a), Ptr::Weak(b)) => a.ptr_eq(b),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum MatchValue {
    Char(char),
    /// `branch` must always be valid
    Split {
        branch: Ptr<State>,
    },
    Wildcard,
}

impl MatchValue {
    /// Assumes MatchValue is either `Char` or `Wildcard`
    fn matches(&self, value: char) -> bool {
        match self {
            MatchValue::Char(c) => *c == value,
            MatchValue::Wildcard => true,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct State {
    value: MatchValue,
    next: Option<Rc<Self>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Regex {
    head: Option<Rc<State>>,
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

#[derive(Debug, Clone, Copy)]
struct Reference<'a>(&'a State);

impl<'a> PartialEq for Reference<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0 as *const _ == other.0 as *const _
    }
}
impl<'a> Eq for Reference<'a> {}

impl<'a> std::hash::Hash for Reference<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.0 as *const _ as usize)
    }
}

#[derive(Debug, Default)]
struct Step<'a> {
    current: HashSet<Reference<'a>>,
    next: HashSet<Reference<'a>>,
    matched: bool,
}

impl<'a> Step<'a> {
    fn add_state(&mut self, state: &'a State) {
        match &state.value {
            MatchValue::Char(_) | MatchValue::Wildcard => {
                self.next.insert(Reference(state));
            }
            MatchValue::Split { branch } => {
                self.add_state(unsafe { branch.as_ref() });

                match &state.next {
                    Some(next) => self.add_state(next),
                    None => self.matched = true,
                }
            }
        };
    }

    fn step(&mut self, to_match: char) {
        std::mem::swap(&mut self.current, &mut self.next);
        self.next.clear();

        let states = self.current.iter().copied().collect::<Vec<_>>();

        for Reference(state) in states {
            // should always be this variant - check if `match ... unreachable!()` is more efficient?
            if state.value.matches(to_match) {
                match &state.next {
                    Some(next) => self.add_state(next),
                    None => self.matched = true,
                }
            }
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
    /// start backwards?
    pub fn from_simple_expression(expression: &str) -> Result<Self, Error> {
        if expression.is_empty() {
            return Ok(Default::default());
        }

        let mut chars = expression.chars().rev();

        let mut next = None;

        while let Some(c) = chars.next() {
            match c {
                '?' => {
                    let value = get_value(chars.next())?;

                    let branch = State {
                        value,
                        next: next.as_ref().map(Rc::clone),
                    };

                    let split = State {
                        value: MatchValue::Split {
                            branch: Ptr::Strong(Rc::new(branch)),
                        },
                        next,
                    };

                    next = Some(Rc::new(split));
                }
                '+' => {
                    let value = get_value(chars.next())?;

                    let state = Rc::new_cyclic(|weak| State {
                        value,
                        next: Some(Rc::new(State {
                            value: MatchValue::Split {
                                branch: Ptr::Weak(Weak::clone(weak)),
                            },
                            next,
                        })),
                    });

                    next = Some(state);
                }
                '*' => {
                    let value = get_value(chars.next())?;

                    let state = Rc::new_cyclic(|weak| State {
                        value,
                        next: Some(Rc::new(State {
                            value: MatchValue::Split {
                                branch: Ptr::Weak(Weak::clone(weak)),
                            },
                            next,
                        })),
                    });

                    next = state.next.as_ref().map(Rc::clone);
                }
                '/' => {
                    let c = chars.next().ok_or(Error::MissingValue)?;

                    let state = State {
                        value: MatchValue::Char(c),
                        next,
                    };

                    next = Some(Rc::new(state));
                }
                '.' => {
                    let state = State {
                        value: MatchValue::Wildcard,
                        next,
                    };

                    next = Some(Rc::new(state));
                }
                _ => {
                    let state = State {
                        value: MatchValue::Char(c),
                        next,
                    };

                    next = Some(Rc::new(state));
                }
            }
        }

        Ok(Self { head: next })
    }

    pub fn test(&self, string: &str) -> bool {
        match &self.head {
            Some(state) => {
                let mut step = Step::default();

                for ch in string.chars() {
                    step.add_state(state);
                    step.step(ch);
                }

                step.matched
            }
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::{MatchValue::*, Ptr, Regex, State};

    #[test]
    fn test_regex_compilation() {
        assert_eq!(
            Regex::from_simple_expression("abc"),
            Ok(Regex {
                head: Some(Rc::new(State {
                    value: Char('a'),
                    next: Some(Rc::new(State {
                        value: Char('b'),
                        next: Some(Rc::new(State {
                            value: Char('c'),
                            next: None,
                        }))
                    }))
                }))
            })
        );

        assert_eq!(
            Regex::from_simple_expression("wha?/"),
            Ok(Regex {
                head: Some(Rc::new(State {
                    value: Char('w'),
                    next: Some(Rc::new(State {
                        value: Char('h'),
                        next: Some(Rc::new(State {
                            value: Char('a'),
                            next: Some(Rc::new(State {
                                value: Char('?'),
                                next: None
                            }))
                        }))
                    }))
                }))
            })
        );
    }

    #[test]
    fn test_simple_execution() {
        let regex = Regex::from_simple_expression("abc").unwrap();
        assert!(regex.test("abc"));
        assert!(!regex.test("acb"));
        assert!(regex.test("my oh myabc oh my"));
        assert!(regex.test("ababcbac"));
        assert!(regex.test("bab abc"));
        assert!(!regex.test("abacbac"));
    }

    #[test]
    fn test_optional() {
        let regex = Regex::from_simple_expression("lbs?");

        assert_eq!(
            regex,
            Ok(Regex {
                head: Some(Rc::new(State {
                    value: Char('l'),
                    next: Some(Rc::new(State {
                        value: Char('b'),
                        next: Some(Rc::new(State {
                            value: Split {
                                branch: Ptr::Strong(Rc::new(State {
                                    value: Char('s'),
                                    next: None
                                }))
                            },
                            next: None
                        }))
                    }))
                }))
            })
        );

        let regex = regex.unwrap();

        assert!(regex.test("lb"));
        assert!(regex.test("lbs"));
        assert!(!regex.test("ls"));

        let regex = Regex::from_simple_expression("allée?s?");

        let next = Rc::new(State {
            value: Split {
                branch: Ptr::Strong(Rc::new(State {
                    value: Char('s'),
                    next: None,
                })),
            },
            next: None,
        });

        assert_eq!(
            regex,
            Ok(Regex {
                head: Some(Rc::new(State {
                    value: Char('a'),
                    next: Some(Rc::new(State {
                        value: Char('l'),
                        next: Some(Rc::new(State {
                            value: Char('l'),
                            next: Some(Rc::new(State {
                                value: Char('é'),
                                next: Some(Rc::new(State {
                                    value: Split {
                                        branch: Ptr::Strong(Rc::clone(&next))
                                    },
                                    next: Some(next)
                                }))
                            }))
                        }))
                    }))
                }))
            })
        )
    }
}
