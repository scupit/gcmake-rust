
// {{MAJOR:3}}-{{MINOR:2}}something{{PATCH}}
// Would resolve to: 100.20something3
// when given version 'v1.2.3'. The number
// after the version name denotes the number of
// zeroes which are added after the version as padding, if needed.

use super::{general_parser::{parse_until_str, ParseSuccess, parse_given_str, point_to_position, parse_letters, parse_next_char, parse_digits}, version_parser::ThreePartVersion};

enum PaddingMethod {
  NoPad,
  EndingZeros(usize)
}

pub fn transform_version(
  version: &ThreePartVersion,
  transformation_str: &str
) -> Result<String, String> {
  return transform_version_helper(version, transformation_str)
    .map_err(|err_msg| err_msg);
}

fn transform_version_helper(
  version: &ThreePartVersion,
  transformation_str: &str
) -> Result<String, String> {
  let mut the_result: String = String::new();
  let mut str_remaining: &str = transformation_str;

  while let Some(ParseSuccess { value, rest: start_of_brace_expression }) = parse_until_str::<String>("{{", str_remaining).unwrap() {
    the_result.push_str(value);

    let (replaced_value, rest) = replace_version_expr(
      version,
      transformation_str,
      start_of_brace_expression
    )?;

    the_result.push_str(&replaced_value);
    str_remaining = rest;
  }

  the_result.push_str(str_remaining);
  return Ok(the_result);
}

fn replace_version_expr<'a>(
  version: &ThreePartVersion,
  full_expression_str: &str,
  start_of_brace_expression: &'a str
) -> Result<(String, &'a str), String> {
  let start_of_inner_expr: &str = parse_given_str::<String>("{{", start_of_brace_expression).unwrap().unwrap().rest;

  match parse_until_str::<String>("}}", start_of_inner_expr).unwrap() {
    None => return Err(format!(
      "The replacement expression was never properly closed with }}.\n{}",
      point_to_position(full_expression_str, start_of_brace_expression)
    )),
    Some(ParseSuccess { value: inner_expr, rest }) => {
      return Ok((
        parse_replace_inner_expr(version, full_expression_str, inner_expr)?,
        &rest[2..]
      ));
    }
  }
}

fn parse_replace_inner_expr(
  ThreePartVersion { major, minor, patch }: &ThreePartVersion,
  full_expression_str: &str,
  start_of_inner_expr: &str
) -> Result<String, String> {
  match parse_letters::<String>(start_of_inner_expr).unwrap() {
    None => return Err(format!(
      "Invalid expression. Must be either MAJOR, MINOR, or PATCH, and can optionally contain a padding expression.\n{}",
      point_to_position(full_expression_str, start_of_inner_expr)
    )),
    Some(ParseSuccess { value, rest }) => {
      let which_value = match value {
        "MAJOR" => major,
        "MINOR" => minor,
        "PATCH" => patch,
        invalid_str => return Err(format!(
          "Invalid version section '{}' specified. Must be either MAJOR, MINOR, or PATCH.\n{}",
          invalid_str,
          point_to_position(full_expression_str, start_of_inner_expr)
        ))
      };

      let mut final_result: String = which_value.to_string();

      match parse_padding_expr(full_expression_str, rest)? {
        PaddingMethod::NoPad => (),
        PaddingMethod::EndingZeros(num_zeros) => {
          for _ in 0..std::cmp::max(0, num_zeros - final_result.len()) {
            final_result += "0";
          }
        }
      };

      Ok(final_result)
    }
  }
}

fn parse_padding_expr(
  full_expression_str: &str,
  padding_expr: &str
) -> Result<PaddingMethod, String> {
  match parse_next_char::<String>(padding_expr).unwrap() {
    None => return Ok(PaddingMethod::NoPad),
    Some(ParseSuccess { value, rest: expr_after_colon }) => {
      if value != ':' {
        return Err(format!(
          "Invalid start to padding expression '{}'. Only a colon ':' can be used to start a padding expression.\n{}",
          value,
          point_to_position(full_expression_str, padding_expr)
        ))
      }
    
      match parse_digits::<String>(expr_after_colon).unwrap() {
        None => return Err(format!(
          "Padding expression should specify a number of padding zeros, but a number was not specified.\n{}",
          point_to_position(full_expression_str, expr_after_colon)
        )),
        Some(ParseSuccess { value: digits, rest }) => {
          if rest.is_empty() {
            return Ok(PaddingMethod::EndingZeros(digits.parse().unwrap()))
          }
          else {
            return Err(format!(
              "Padding expression should end with the digits, but contains the additional string \"{}\".\n{}",
              rest,
              point_to_position(full_expression_str, expr_after_colon)
            ));
          }
        }
      }
    }
  }
}