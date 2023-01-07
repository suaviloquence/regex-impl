use std::fmt;

use crate::tokenize::{self, MatchCharacter, Repeat, Token};

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

#[derive(Clone, Default, PartialEq)]
pub struct Regex {
	states: Vec<State>,
	head: usize,
	beginning_boundary: bool,
	end_boundary: bool,
}

impl fmt::Debug for Regex {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"Regex {{\n head: {},\n ^: {},\n $: {},\n states: [\n",
			self.head, self.beginning_boundary, self.end_boundary
		)?;

		for (i, state) in self.states.iter().enumerate().rev() {
			write!(f, "  {i}: {state:?},\n")?;
		}

		write!(f, " ]\n}}")
	}
}

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

impl Regex {
	pub fn from_simple_expression(expression: &str) -> tokenize::Result<Self> {
		Token::tokenize_regex(expression).map(|toks| Self::from_tokens(&toks))
	}

	fn convert_tokens(tokens: &[Box<Token>], states: &mut Vec<State>, index: &mut usize) {
		macro_rules! push {
			($state: expr) => {
				states.push($state);
				*index += 1;
			};
		}

		for token in tokens.into_iter().rev() {
			let next = *index;
			if let Repeat::AtLeast(_) = token.repeat {
				push!(State {
					// fill later
					value: MatchValue::Split { branch: 0 },
					next,
				});
			}

			match &token.value {
				MatchCharacter::Char(c) => {
					push!(State {
						value: MatchValue::Char(*c),
						next: *index
					});
				}
				MatchCharacter::Wildcard => {
					push!(State {
						value: MatchValue::Wildcard,
						next: *index
					});
				}
				MatchCharacter::String(tokens) => Self::convert_tokens(tokens, states, index),
				MatchCharacter::Or(_, _) => todo!(),
				MatchCharacter::Beginning | MatchCharacter::End => {
					unreachable!("Regex boundary in convert_tokens")
				}
			}

			if let Repeat::AtLeast(_) = token.repeat {
				states[dbg!(next + 1)].next = *index;
			}

			if let Repeat::Optional | Repeat::AtLeast(0) = token.repeat {
				push!(State {
					value: MatchValue::Split { branch: next },
					next: *index,
				});
			}
		}
	}

	fn from_tokens(mut tokens: &[Box<Token>]) -> Self {
		let mut states = vec![State {
			next: 0,
			value: MatchValue::Match,
		}];
		let mut index = 0;

		let beginning_boundary = matches!(
			tokens.first().map(|x| x.value == MatchCharacter::Beginning),
			Some(true),
		);

		if beginning_boundary {
			tokens = &tokens[1..];
		}

		let end_boundary = matches!(
			tokens.last().map(|x| x.value == MatchCharacter::End),
			Some(true)
		);

		if end_boundary {
			tokens = &tokens[..tokens.len() - 1];
		}

		Self::convert_tokens(&tokens, &mut states, &mut index);

		Self {
			head: index,
			states,
			beginning_boundary,
			end_boundary,
		}
	}

	pub fn test(&self, string: &str) -> bool {
		let mut step = Step::new(&self.states);

		if self.beginning_boundary {
			step.add_state(self.head);
		}

		for ch in string.chars() {
			if !self.beginning_boundary {
				step.add_state(self.head);
			}

			if !self.end_boundary {
				step.matched = false;
			}

			step.step(ch);

			println!("{:?}", step);
		}

		step.matched
	}
}
