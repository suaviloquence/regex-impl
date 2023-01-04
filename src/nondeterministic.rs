use std::collections::HashSet;

#[derive(Debug, Clone)]
enum MatchValue {
	Char(char),
	Split { branch: Box<State> },
	// TODO: Matched
}

#[derive(Debug, Clone)]
struct State {
	value: MatchValue,
	next: Option<Box<Self>>,
}

#[derive(Debug, Clone, Default)]
pub struct Regex {
	head: Option<State>,
}

#[derive(Debug, Clone)]
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
			MatchValue::Char(_) => {
				self.next.insert(Reference(state));
			}
			MatchValue::Split { branch } => {
				self.add_state(branch);

				match &state.next {
					Some(next) => self.add_state(next),
					None => self.matched = true,
				}
			}
		};
	}

	fn step(&mut self, to_match: char) {
		std::mem::swap(&mut self.current, &mut self.next);
		let states = self.current.iter().copied().collect::<Vec<_>>();

		for Reference(state) in states {
			// should always be this variant - check if `match ... unreachable!()` is more efficient?
			if let MatchValue::Char(c) = &state.value {
				if *c == to_match {
					match &state.next {
						Some(next) => self.add_state(next),
						None => self.matched = true,
					}
				}
			}
		}
	}
}

impl Regex {
	/// start backwards?
	pub fn from_simple_expression(expression: &str) -> Result<Self, Error> {
		if expression.is_empty() {
			return Ok(Default::default());
		}

		let mut chars = expression.chars();

		let mut head = State {
			value: MatchValue::Char('\0'),
			next: None,
		};

		let mut state = &mut head;

		while let Some(c) = chars.next() {
			match c {
				'?' | '+' | '*' => {
					todo!()
				}
				_ => {
					state.next = Some(Box::new(State {
						value: MatchValue::Char(c),
						next: None,
					}));

					state = state.next.as_mut().unwrap();
				}
			}
		}

		Ok(Self {
			head: head.next.map(|b| *b),
		})
	}

	pub fn test(&self, string: &str) -> bool {
		match &self.head {
			Some(state) => {
				let mut step = Step::default();

				step.add_state(state);

				todo!()
			}
			None => true,
		}
	}
}
