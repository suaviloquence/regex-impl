use std::fmt;

#[derive(Debug, PartialEq)]
pub enum MatchCharacter {
	Char(char),
	Wildcard,
	String(Box<[Box<Token>]>),
	/// can only occur as the first element of the top-level array - `Token.repeat` is ignored for this
	Beginning,
	/// can only occur as the last element of the top-level array - `Token.repeat` is ignored for this
	End,
	Or(Box<Token>, Box<Token>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Repeat {
	Optional,
	/// assumed to be more than 0
	Exactly(usize),
	AtLeast(usize),
	/// assumed to be more than 0
	AtMost(usize),
	Range(usize, usize),
}

#[derive(Debug, PartialEq)]
pub struct Token {
	pub repeat: Repeat,
	pub value: MatchCharacter,
}

#[derive(Debug, PartialEq)]
pub enum Error {
	MissingToken { at: usize },
	InvalidModifierLocation { at: usize },
	MismatchedGroup { at: usize },
	UnexpectedBoundary { at: usize },
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidModifierLocation { at } => {
				write!(f, "Invalid modifier (+, ?, etc.) at char {}", at)
			}
			Self::MismatchedGroup { at } => {
				write!(f, "Mismatched group (parentheses) at char {}", at)
			}
			Self::MissingToken { at } => write!(f, "Missing token at char {}", at),
			Self::UnexpectedBoundary { at } => {
				write!(f, "Unexpected expression boundary (^ or $) at char {}", at)
			}
		}
	}
}

impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;

impl Token {
	pub fn tokenize_regex(expression: &str) -> Result<Vec<Box<Self>>> {
		let mut vec = Vec::new();

		let chars: Vec<_> = expression.chars().collect();

		let mut i = 0;

		if let Some('^') = chars.first() {
			vec.push(Box::new(Token {
				repeat: Repeat::Exactly(1),
				value: MatchCharacter::Beginning,
			}));
			i += 1;
		}

		let mut end = chars.len();

		if let Some('$') = chars.last() {
			end -= 1;
		}

		while i < end {
			vec.push(Self::tokenize_one(&chars, &mut i)?);
		}

		if end < chars.len() {
			vec.push(Box::new(Token {
				repeat: Repeat::Exactly(1),
				value: MatchCharacter::End,
			}))
		}

		Ok(vec)
	}

	fn tokenize_one(chars: &[char], i: &mut usize) -> Result<Box<Self>> {
		if *i >= chars.len() {
			return Err(Error::MissingToken { at: *i });
		}

		let value = match chars[*i] {
			// TODO check valid escape sequences
			'\\' => {
				*i += 1;
				MatchCharacter::Char(
					*chars
						.get(*i)
						.ok_or_else(|| Error::MissingToken { at: *i })?,
				)
			}
			'^' | '$' => return Err(Error::UnexpectedBoundary { at: *i }),
			'?' | '*' | '+' => return Err(Error::InvalidModifierLocation { at: *i }),
			'(' => {
				*i += 1;
				let mut vec = Vec::new();

				// TODO check for correct i handling at boundaries
				loop {
					if *i >= chars.len() {
						return Err(Error::MismatchedGroup { at: *i });
					}

					if chars[*i] == ')' {
						break;
					}

					vec.push(Self::tokenize_one(chars, i)?);
				}

				MatchCharacter::String(vec.into_boxed_slice())
			}
			')' => {
				*i += 1;
				return Err(Error::MismatchedGroup { at: *i });
			}
			'.' => MatchCharacter::Wildcard,
			'|' => todo!(),
			ch => MatchCharacter::Char(ch),
		};

		*i += 1;

		let repeat = match chars.get(*i) {
			Some('?') => Repeat::Optional,
			Some('+') => Repeat::AtLeast(1),
			Some('*') => Repeat::AtLeast(0),
			_ => {
				// don't consume
				*i -= 1;
				Repeat::Exactly(1)
			}
		};
		*i += 1;

		Ok(Box::new(Self { repeat, value }))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use MatchCharacter::*;
	use Repeat::*;

	macro_rules! Tk {
		($r: expr, $v: expr) => {
			Box::new(Token {
				repeat: $r,
				value: $v,
			})
		};
	}

	#[test]
	fn test_tokenize() {
		assert_eq!(
			Token::tokenize_regex("abcd"),
			Ok(vec![
				Tk!(Exactly(1), Char('a')),
				Tk!(Exactly(1), Char('b')),
				Tk!(Exactly(1), Char('c')),
				Tk!(Exactly(1), Char('d'))
			])
		);

		assert_eq!(
			Token::tokenize_regex("a(b(cd)?)+"),
			Ok(vec![
				Tk!(Exactly(1), Char('a')),
				Tk!(
					AtLeast(1),
					String(
						vec![
							Tk!(Exactly(1), Char('b')),
							Tk!(
								Optional,
								String(
									vec![Tk!(Exactly(1), Char('c')), Tk!(Exactly(1), Char('d'))]
										.into_boxed_slice()
								)
							)
						]
						.into_boxed_slice()
					)
				)
			])
		)
	}
}
