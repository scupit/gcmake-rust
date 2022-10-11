use std::fmt::Write;

pub struct ParseSuccess<'a, T> {
  pub value: T,
  pub rest: &'a str
}

#[derive(Debug)]
pub enum ParseError<E> {
  NoneMatched {
    what_parsing: String,
    parsed_from: String,
    failure_reason: String
  },
  InvalidIdentifier {
    what_parsing: String,
    identifier: String,
    expected: Option<String>,
    parsed_from: String
  },
  Custom(E)
}

pub type ParseResult<'a, T, E> = Result<Option<ParseSuccess<'a, T>>, ParseError<E>>;
pub type ParserFunc<'a, T, E> = fn(&str) -> ParseResult<'a, T, E>;

pub trait Parser<'a, T, E> {
  fn parse(&self, s: &'a str) -> ParseResult<'a, T, E>;
}

impl<'a, T, E, F> Parser<'a, T, E> for F
  where F: Fn(&'a str) -> ParseResult<'a, T, E>
{
  fn parse(&self, s: &'a str) -> ParseResult<'a, T, E> {
    self(s)
  }
}

pub struct ParserWrapper<'a, T, E>(pub ParserFunc<'a, T, E>);

impl<'a, T, E> Parser<'a, T, E> for ParserWrapper<'a, T, E> {
  fn parse(&self, s: &str) -> ParseResult<'a, T, E> {
    (self.0)(s)
  }
}

pub fn alternatives_parse<'a, T, E>(
  s: &'a str,
  parsers: Vec<&dyn Parser<'a, T, E>>
) -> ParseResult<'a, T, E> {
  for parser in parsers {
    match parser.parse(s) {
      Ok(Some(success)) => return Ok(Some(success)),
      Ok(None) => continue,
      Err(err) => return Err(err)
    }
  }

  Ok(None)
}

pub fn parse_whitespace<'a, E>(s: &'a str) -> ParseResult<'a, (), E> {
  return Ok(Some(ParseSuccess {
    value: (),
    rest: s.trim_start()
  }));
}

pub fn parse_letters<'a, E>(s: &'a str) -> ParseResult<'a, &'a str, E> {
  for (index, some_char) in s.char_indices() {
    match some_char {
      'a'..='z' | 'A'..='Z' => continue,
      _ => {
        if index == 0 {
          return Ok(None);
        }
        else {
          return Ok(Some(ParseSuccess {
            value: &s[..index],
            rest: &s[index..]
          }))
        }
      }
    }
  }

  return Ok(Some(ParseSuccess {
    value: s,
    rest: ""
  }));
}

pub fn parse_given_str<'a, E>(
  str_parsing_for: &'a str,
  s: &'a str
) -> ParseResult<'a, &'a str, E> {
  if str_parsing_for.is_empty() || str_parsing_for.len() > s.len() {
    return Ok(None);
  }

  let char_matcher = str_parsing_for
    .chars()
    .zip(s[..str_parsing_for.len()].chars());

  for (looking_for, actual) in char_matcher {
    if looking_for != actual {
      return Ok(None);
    }
  }

  return Ok(Some(ParseSuccess {
    value: str_parsing_for.clone(),
    rest: &s[str_parsing_for.len()..]
  }))
}

pub fn parse_next_char<'a, E>(s: &'a str) -> ParseResult<'a, char, E> {
  if s.is_empty() {
    return Ok(None);
  }
  else {
    return Ok(Some(ParseSuccess {
      value: s.chars().nth(0).unwrap(),
      rest: &s[1..]
    }))
  }
}

pub fn parse_until_str<'a, E>(
  stopper_str: &str,
  s: &'a str
) -> ParseResult<'a, &'a str, E> {
  let mut offset: usize = 0;

  while stopper_str.len() + offset <= s.len() {
    if &s[offset..offset + stopper_str.len()] == stopper_str {
      return Ok(Some(ParseSuccess {
        value: &s[..offset],
        rest: &s[offset..]
      }));
    }

    offset += 1;
  }

  return Ok(None)
}

pub fn parse_digits<'a, E>(s: &'a str) -> ParseResult<'a, &'a str, E> {
  let mut digit_count: usize = 0;

  for maybe_digit in s.chars() {
    match maybe_digit {
      '0'..='9' => digit_count += 1,
      _ => break
    }
  }

  return if digit_count == 0 {
    Ok(None)
  }
  else {
    Ok(Some(ParseSuccess {
      value: &s[..digit_count],
      rest: &s[digit_count..]
    }))
  }
}

pub fn parse_given_str_after_whitespace<'a, E>(
  str_parsing_for: &'a str,
  s: &'a str
) -> ParseResult<'a, &'a str, E> {
  let rest = parse_whitespace(s)?.unwrap().rest;
  return parse_given_str(str_parsing_for, rest);
}

pub fn point_to_position(full_str: &str, parsed_from: &str) -> String {
  let index = full_str.rfind(parsed_from.trim_start()).unwrap();
  let mut resulting_string: String = String::new();

  writeln!(resulting_string, "\t{}", full_str).unwrap();
  write!(resulting_string, "\t").unwrap();
  for _ in 0..index {
    write!(resulting_string, " ").unwrap();
  }
  // println!("\\-- Error occurs here");
  writeln!(resulting_string, "^-- Error occurs here").unwrap();
  return resulting_string;
}
