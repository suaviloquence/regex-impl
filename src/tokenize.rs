#[derive(Debug, PartialEq)]
pub enum MatchCharacter {
    Char(char),
    Wildcard,
    String(Box<[Box<Token>]>),
    Or(Box<Token>, Box<Token>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Repeat {
    Optional,
    Any,
    Exactly(usize),
    AtLeast(usize),
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
}

impl Token {
    pub fn tokenize_regex(expression: &str) -> Result<Vec<Box<Self>>, Error> {
        let mut vec = Vec::new();
        let chars: Vec<_> = expression.chars().collect();
        let mut i = 0;
        Self::tokenize(&chars, &mut i, &mut vec, true)?;
        Ok(vec)
    }

    fn tokenize(
        chars: &[char],
        i: &mut usize,
        vec: &mut Vec<Box<Self>>,
        is_top: bool,
    ) -> Result<(), Error> {
        if *i >= chars.len() {
            return Ok(());
        }
        let tok = match chars[*i] {
            '\\' => {
                *i += 1;
                MatchCharacter::Char(
                    *chars
                        .get(*i)
                        .ok_or_else(|| Error::MissingToken { at: *i })?,
                )
            }
            '?' | '*' | '+' => return Err(Error::InvalidModifierLocation { at: *i }),
            '(' => {
                *i += 1;
                let mut vec = Vec::new();

                // TODO check for correct i handling at boundaries
                Self::tokenize(chars, i, &mut vec, false)?;

                MatchCharacter::String(vec.into_boxed_slice())
            }
            ')' => {
                *i += 1;
                if is_top {
                    return Err(Error::MissingToken { at: *i });
                } else {
                    return Ok(());
                }
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

        vec.push(Box::new(Self { repeat, value: tok }));

        Self::tokenize(chars, i, vec, is_top)
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
    }
}
