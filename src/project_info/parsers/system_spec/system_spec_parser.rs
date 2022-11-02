use std::{collections::{HashSet, BTreeSet}};

use crate::project_info::parsers::general_parser::{self, parse_given_str};
use colored::Colorize;
use general_parser::{ParseResult, parse_whitespace, ParseError, ParseSuccess, alternatives_parse, parse_given_str_after_whitespace, point_to_position, Parser};

const VALID_JOINT_TERMS: [&'static str; 2] = ["and", "or"];

pub type SystemSpecParseResult<'a> = ParseResult<'a, SystemSpecExpressionTree, SpecParseError>;
struct SystemSpecParseOptions<'a> {
  valid_feature_names: Option<HashSet<&'a str>>
}

#[derive(PartialEq, Eq, Clone)]
pub enum SingleSystemSpec {
  // Target systems
  Android,
  Windows,
  Linux,
  MacOS,
  Unix,

  // Compilers and compiler environments
  MinGW,
  GCC,
  Clang,
  MSVC,
  Emscripten
}

impl SingleSystemSpec {
  fn is_valid_spec_str(spec_str: &str) -> bool {
    return Self::from_str(spec_str).is_some();
  }

  // TODO: Refactor these somehow
  fn from_str(spec_str: &str) -> Option<Self> {
    let valid_spec: Self = match spec_str {
      "android" => Self::Android,
      "windows" => Self::Windows,
      "linux" => Self::Linux,
      "macos" => Self::MacOS,
      "unix" => Self::Unix,

      "mingw" => Self::MinGW,
      "gcc" => Self::GCC,
      "clang" => Self::Clang,
      "msvc" => Self::MSVC,
      "emscripten" => Self::Emscripten,
      _ => return None
    };

    return Some(valid_spec);
  }

  pub fn to_str(&self) -> &str {
    match self {
      Self::Android => "android",
      Self::Windows => "windows",
      Self::Linux => "linux",
      Self::MacOS => "macos",
      Self::Unix => "unix",

      Self::MinGW => "mingw",
      Self::GCC => "gcc",
      Self::Clang => "clang",
      Self::MSVC => "msvc",
      Self::Emscripten => "emscripten"
    }
  }
}

/*
  Examples:
    - ((not windows and linux))
    - ((windows and not (mingw or unix)))
*/

// FIXME: Need to add precedence and simplification.
// ((not windows and not linux)) currently resolves to ((not (windows and not linux))) instead
// of the desired (( (not windows) and (not linux) ))).
#[derive(Clone)]
pub enum SystemSpecExpressionTree {
  Value(SingleSystemSpec),
  Feature {
    name: String
  },
  Not(Box<SystemSpecExpressionTree>),
  And(Box<SystemSpecExpressionTree>, Box<SystemSpecExpressionTree>),
  Or(Box<SystemSpecExpressionTree>, Box<SystemSpecExpressionTree>),
  ParenGroup(Box<SystemSpecExpressionTree>)
}

impl SystemSpecExpressionTree {
  pub fn parse_from<'a, 'b>(
    s: &'a str,
    maybe_feature_name_list: Option<impl Iterator<Item = &'b str>>
  ) -> SystemSpecParseResult<'a> {

    let options: SystemSpecParseOptions = SystemSpecParseOptions {
      valid_feature_names: maybe_feature_name_list
        .map(|names_iter| names_iter.collect())
    };

    return parse_full_spec(s, &options);
    // match parse_full_spec(s) {
    //   Ok(Some(ParseSuccess { value, rest })) => {
    //     // TODO: Reorder the tree to comply with precedence, then simplify out equivalence expressions.
    //     // Also need a custom equals function.
    //     // when simplified(L) and simplified(R)
    //     //  - if L is equivalent to R, simplify to either.
    //     //  - if L is equivalent to !R
    //   },
    //   unprocessable_result => unprocessable_result
    // }
  }

  fn inner_to_string(&self) -> String {
    match self {
      Self::Value(single_spec) => single_spec.to_str().to_string(),
      Self::Feature { name } => format!("feature:{}", name),
      Self::Not(expr) => format!("not {}", expr.inner_to_string()),
      Self::And(left_expr, right_expr) => format!("{} and {}", left_expr.inner_to_string(), right_expr.inner_to_string()),
      Self::Or(left_expr, right_expr) => format!("{} or {}", left_expr.inner_to_string(), right_expr.inner_to_string()),
      Self::ParenGroup(expr) => format!("({})", expr.inner_to_string())
    }
  }

  fn exactly_matches_structure(&self, other: &SystemSpecExpressionTree) -> bool {
    match (self, other) {
      (SystemSpecExpressionTree::Value(value), SystemSpecExpressionTree::Value(other_val)) => value == other_val,
      (SystemSpecExpressionTree::Feature { name: this_name }, SystemSpecExpressionTree::Feature { name: other_name }) => this_name == other_name,
      (SystemSpecExpressionTree::ParenGroup(group), SystemSpecExpressionTree::ParenGroup(other_group)) => {
        group.exactly_matches_structure(other_group)
      },
      (SystemSpecExpressionTree::Not(expr), SystemSpecExpressionTree::Not(other_expr)) => {
        expr.exactly_matches_structure(other_expr)
      },
      (
        SystemSpecExpressionTree::Or(l_expr, r_expr),
        SystemSpecExpressionTree::Or(l_other_expr, r_other_expr)
      ) => {
        l_expr.exactly_matches_structure(l_other_expr) && r_expr.exactly_matches_structure(r_other_expr)
      },
      (
        SystemSpecExpressionTree::And(l_expr, r_expr),
        SystemSpecExpressionTree::And(l_other_expr, r_other_expr)
      ) => {
        l_expr.exactly_matches_structure(l_other_expr) && r_expr.exactly_matches_structure(r_other_expr)
      },
      _ => false
    }
  }
}

impl ToString for SystemSpecExpressionTree {
  fn to_string(&self) -> String {
    format!(
      "(({}))",
      self.inner_to_string()
    )
  }
}

#[derive(Debug)]
pub enum SpecParseError {
  NotClosed {
    what_parsing: String,
    parsed_from: String,
    needed: String
  },
  InvalidFeatureName {
    parsed_from: String,
    received: String,
    valid_names: BTreeSet<String>
  },
  _NotOpened {
    what_parsing: String,
    parsed_from: String,
    needed: String
  }
}

pub fn parse_spec_with_diagnostic<'a>(
  expr_str: &'a str,
  maybe_feature_name_list: Option<&Vec<&str>>
) -> Result<Option<ParseSuccess<'a, SystemSpecExpressionTree>>, String> {
  let feature_list_iter: Option<_> = maybe_feature_name_list
    .map(|feature_list| feature_list.iter().copied());

  return match SystemSpecExpressionTree::parse_from(expr_str, feature_list_iter) {
    Ok(None) => Ok(None),
    Ok(Some(success_data)) => Ok(Some(success_data)),
    Err(parsing_err) => match parsing_err {
      ParseError::InvalidIdentifier { what_parsing, identifier, expected, parsed_from } => {
        let expected_string: String = expected
          .map_or(
            String::from(""),
            |the_expected| format!(" (expected {})", the_expected)
          );

        Err(format!(
          "Failed to parse {} due to invalid identifier '{}'{}.\n{}",
          what_parsing,
          identifier,
          expected_string,
          point_to_position(expr_str, &parsed_from)
        ))
      },
      ParseError::NoneMatched { what_parsing, parsed_from, failure_reason } => {
        Err(format!(
          "Failed to parse {} because {}.\n{}",
          what_parsing,
          failure_reason,
          point_to_position(expr_str, &parsed_from)
        ))
      },
      ParseError::Custom(custom_err) => match custom_err {
        SpecParseError::NotClosed { what_parsing, parsed_from, needed } => {
          Err(format!(
            "Failed to parse {} because it wasn't properly closed with '{}'.\n{}",
            what_parsing,
            needed,
            point_to_position(expr_str, &parsed_from)
          ))
        },
        SpecParseError::InvalidFeatureName { parsed_from, received, valid_names } => {
          let valid_names_str: String = valid_names.into_iter()
            .collect::<Vec<String>>()
            .join(", ");

          Err(format!(
            "Failed to parse feature spec because it specifies the name \"{}\" which isn't in the specified features list. Must be one of: {}.\n{}",
            received.yellow(),
            valid_names_str.green(),
            point_to_position(expr_str, &parsed_from)
          ))
        },
        SpecParseError::_NotOpened { what_parsing, parsed_from, needed } => {
          unreachable!(
            "Failed to parse {} because it was never properly opened with '{}'. NOTE that this branch only exists for parser debugging purposes, and should be unreachable when not debugging the parser.\n{}.",
            what_parsing,
            needed,
            point_to_position(expr_str, &parsed_from)
          );
          // Err(format!(
          //   "Failed to parse {} because it was never properly opened with '{}'.\n{}",
          //   what_parsing,
          //   needed,
          //   point_to_position(expr_str, &parsed_from)
          // ))
        }
      }
    }
  }
}

fn parse_full_spec<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  return match parse_given_str_after_whitespace::<SpecParseError>("((", s)? {
    // None => Err(ParseError::Custom(SpecParseError::NotOpened {
    //   what_parsing: String::from("start of full system spec expression"),
    //   parsed_from: s.to_string(),
    //   needed: String::from("((")
    // })),
    None => Ok(None),
    Some(ParseSuccess { value: _, rest }) => match parse_expression(rest, options)? {
      None => Ok(None),
      Some(ParseSuccess { value: expr, rest: after_inner }) => match parse_given_str_after_whitespace::<SpecParseError>("))", after_inner) {
        Ok(Some(ParseSuccess { value: _, rest: after_full_spec })) => Ok(Some(ParseSuccess {
          value: expr,
          rest: after_full_spec.trim_start()
        })),
        // _ => Ok(None)
        _ => Err(ParseError::Custom(SpecParseError::NotClosed {
          what_parsing: String::from("full system spec expression"),
          parsed_from: dbg!(after_inner.to_string()),
          needed: String::from("))")
        }))
      }
    }
  }
}

fn parse_paren_group<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  match parse_given_str_after_whitespace::<SpecParseError>("(", s).unwrap() {
    None => Ok(None),
    Some(ParseSuccess { value: _, rest}) => match parse_expression(rest, options)? {
      None => Err(ParseError::NoneMatched {
        what_parsing: String::from("parenthesized expression group"),
        parsed_from: rest.to_string(),
        failure_reason: String::from("The group does not contain a valid expression.")
      }),
      Some(ParseSuccess { value: grouped_expr, rest: after_contained_expr }) => match parse_given_str_after_whitespace::<SpecParseError>(")", after_contained_expr) {
        Ok(Some(ParseSuccess { value: _, rest: after_group_end })) => Ok(Some(ParseSuccess {
          value: SystemSpecExpressionTree::ParenGroup(Box::new(grouped_expr)),
          rest: after_group_end
        })),
        _ => Ok(None)
      }
    }
  }
}

fn parse_without_joint<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  return alternatives_parse(
    s,
    options,
    vec![&parse_not, &parse_paren_group, &parse_feature, &parse_value]
  );
}

fn parse_without_or<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  return alternatives_parse(
    s,
    options,
    vec![&parse_inner_and, &parse_not_without_or, &parse_paren_group, &parse_feature, &parse_value]
  );
}

fn parse_without_and<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  return alternatives_parse(
    s,
    options,
    vec![&parse_inner_or, &parse_not_without_and, &parse_paren_group, &parse_feature, &parse_value]
  );
}

fn parse_expression<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  return alternatives_parse(
    s,
    options,
    vec![&parse_and, &parse_or, &parse_not, &parse_paren_group, &parse_feature, &parse_value]
  );
}

fn parse_not_base<'a, 'b>(
  s: &'a str,
  options: &'b SystemSpecParseOptions,
  inner_expression_parser: &dyn Parser<'a, SystemSpecExpressionTree, SystemSpecParseOptions<'b>, SpecParseError>
) -> SystemSpecParseResult<'a> {
  return match parse_token(s, true)? {
    None => Ok(None),
    Some(ParseSuccess { value, rest }) => match value {
      "not" => match inner_expression_parser.parse(rest, options)? {
        None => Ok(None),
        Some(ParseSuccess { value: expr, rest: rest_of_expr }) => {
          Ok(Some(ParseSuccess {
            value: SystemSpecExpressionTree::Not(Box::new(expr)),
            rest: rest_of_expr
          }))
        }
      },
      _ => Ok(None)
    }
  }
  
}

fn parse_not<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  parse_not_base(s, options, &parse_expression)
}

fn parse_not_without_and<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  parse_not_base(s, options, &parse_without_and)
}

fn parse_not_without_or<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  parse_not_base(s, options, &parse_without_or)
}

fn parse_value<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  return match parse_token(s, true)? {
    None => Ok(None),
    Some(ParseSuccess { value, rest }) => match SingleSystemSpec::from_str(value) {
      Some(single_spec) => Ok(Some(ParseSuccess {
        value: SystemSpecExpressionTree::Value(single_spec),
        rest
      })),
      None => Err(ParseError::InvalidIdentifier {
        what_parsing: String::from("single system specifier"),
        identifier: value.to_string(),
        expected: None,
        parsed_from: s.to_string()
      })
    }
  }
}

fn parse_feature<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  return match parse_token(s, true)? {
    None => Ok(None),
    Some(ParseSuccess { value: left_token, rest: after_left }) => match left_token {
      "feature" => match parse_given_str(":", after_left)? {
        None => Err(ParseError::InvalidIdentifier {
          what_parsing: String::from("feature specifier separator ':'"),
          identifier: after_left[0..1].to_string(),
          expected: Some(String::from(":")),
          parsed_from: after_left.to_string()
        }),
        Some(ParseSuccess { value: _, rest: after_separator }) => match parse_token(after_separator, false)? {
          Some(ParseSuccess { value, rest }) => {
            if let Some(valid_feature_names) = &options.valid_feature_names {
              if !valid_feature_names.contains(value) {
                return Err(ParseError::Custom(SpecParseError::InvalidFeatureName {
                  parsed_from: after_separator.to_string(),
                  received: value.to_string(),
                  valid_names: valid_feature_names.clone().into_iter()
                    .map(|feature_name| feature_name.to_string())
                    .collect()
                }))
              }
            }

            return Ok(Some(ParseSuccess {
              value: SystemSpecExpressionTree::Feature { name: value.to_string() },
              rest
            }))
          },
          None => Err(ParseError::NoneMatched {
            what_parsing: String::from("feature name"),
            failure_reason: String::from("A feature name was not provided."),
            parsed_from: after_separator.to_string()
          })
        }
      },
      _ => Ok(None)
    }
  }
}

fn parse_inner_and<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  parse_joined_term(
    &parse_without_joint,
    "and",
    SystemSpecExpressionTree::And,
    s,
    options
  )
}

fn parse_and<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  parse_joined_term(
    &parse_without_and,
    "and",
    SystemSpecExpressionTree::And,
    s,
    options
  )
}

fn parse_inner_or<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  parse_joined_term(
    &parse_without_joint,
    "or",
    SystemSpecExpressionTree::Or,
    s,
    options
  )
}

fn parse_or<'a>(
  s: &'a str,
  options: &SystemSpecParseOptions
) -> SystemSpecParseResult<'a> {
  parse_joined_term(
    &parse_without_or,
    "or",
    SystemSpecExpressionTree::Or,
    s,
    options
  )
}

fn parse_joined_term<'a, 'b, F>(
  left_side_parser: &dyn Parser<'a, SystemSpecExpressionTree, SystemSpecParseOptions<'b>, SpecParseError>,
  join_word: &str,
  joint_constructor: F,
  s: &'a str,
  options: &SystemSpecParseOptions<'b>
) -> SystemSpecParseResult<'a>
  where F: Fn(Box<SystemSpecExpressionTree>, Box<SystemSpecExpressionTree>) -> SystemSpecExpressionTree
{
  assert!(
    VALID_JOINT_TERMS.contains(&join_word),
    "When parsing a joined term (i.e. 'and' or 'or' expression), the joining word must be present in VALID_JOINT_TERMS."
  );

  match left_side_parser.parse(s, options)? {
    None => Ok(None),
    Some(ParseSuccess { value: left_expr, rest: after_left }) => match parse_token(after_left, true)? {
      Some(ParseSuccess { value: middle_token, rest: after_middle_token }) => match parse_expression(after_middle_token, options)? {
        None => Ok(None),
        Some(ParseSuccess { value: right_expr, rest }) => {
          if middle_token == join_word {
            Ok(Some(ParseSuccess {
              value: (joint_constructor)(Box::new(left_expr), Box::new(right_expr)),
              rest
            }))
          }
          else if !VALID_JOINT_TERMS.contains(&middle_token) {
            let valid_term_list = VALID_JOINT_TERMS
              .iter()
              .map(|valid_middle_term_str| format!("'{}'", valid_middle_term_str))
              .collect::<Vec<String>>()
              .join(",");

            Err(ParseError::InvalidIdentifier {
              what_parsing: format!("joint '{}' expression", join_word),
              identifier: middle_token.to_string(),
              expected: Some(format!("one of: {}", valid_term_list)),
              parsed_from: after_left.to_string()
            }) 
          }
          else {
            Ok(None)
          }
        }
      },
      _ => Ok(None)
    }
  }
}

fn parse_token<'a>(
  s: &'a str,
  should_parse_whitespace_first: bool
) -> ParseResult<'a, &'a str, SpecParseError> {
  let str_parsing: &str = if should_parse_whitespace_first
    { parse_whitespace(s)?.unwrap().rest }
    else { s };

  let mut non_token_char_index: usize = 0;

  for c in str_parsing.chars() {
    match c {
      'a'..='z' | 'A'..='Z' | '-' | '_' => non_token_char_index += 1,
      ' ' | '(' | ')' | ':' => break,
      invalid_token => return Err(ParseError::InvalidIdentifier {
        what_parsing: String::from("token"),
        identifier: invalid_token.to_string(),
        expected: None,
        parsed_from: str_parsing[non_token_char_index..].to_string()
      })
    }
  }

  let token: &'a str = &str_parsing[..non_token_char_index];

  if token.is_empty() {
    return Ok(None)
  }

  return Ok(Some(ParseSuccess {
    value: token,
    rest: &str_parsing[non_token_char_index..]
  }));
}

struct ParserTestGroup<'a> {
  raw_expr: &'a str,
  expected_tree: Option<SystemSpecExpressionTree>
}

#[test]
fn test_parser() {
  let valid_expressions: Vec<ParserTestGroup<'_>> = vec![
    ParserTestGroup {
      raw_expr: "((windows))",
      expected_tree: Some(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows))
    },
    ParserTestGroup {
      raw_expr: "((windows or linux))",
      expected_tree: Some(SystemSpecExpressionTree::Or(
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows)),
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Linux))
      ))
    },
    ParserTestGroup {
      raw_expr: "((windows and linux))",
      expected_tree: Some(SystemSpecExpressionTree::And(
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows)),
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Linux))
      ))
    },
    ParserTestGroup {
      raw_expr: "((windows and not linux))",
      expected_tree: Some(SystemSpecExpressionTree::And(
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows)),
        Box::new(SystemSpecExpressionTree::Not(
          Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Linux))
        ))
      ))
    },
    ParserTestGroup {
      raw_expr: "((windows and not not linux))",
      expected_tree: Some(SystemSpecExpressionTree::And(
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows)),
        Box::new(SystemSpecExpressionTree::Not(
          Box::new(SystemSpecExpressionTree::Not(
            Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Linux))
          ))
        ))
      ))
    },
    ParserTestGroup {
      raw_expr: "((mingw and not (linux or macos))) some text after",
      expected_tree: Some(SystemSpecExpressionTree::And(
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::MinGW)),
        Box::new(SystemSpecExpressionTree::Not(
          Box::new(SystemSpecExpressionTree::ParenGroup(
            Box::new(SystemSpecExpressionTree::Or(
              Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Linux)),
              Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::MacOS))
            ))
          ))
        ))
      ))
    },
    ParserTestGroup {
      raw_expr: "((windows or linux or mingw))",
      expected_tree: Some(SystemSpecExpressionTree::Or(
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows)),
        Box::new(SystemSpecExpressionTree::Or(
          Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Linux)),
          Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::MinGW))
        ))
      ))
    },
    ParserTestGroup {
      raw_expr: "((windows and linux and mingw))",
      expected_tree: Some(SystemSpecExpressionTree::And(
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows)),
        Box::new(SystemSpecExpressionTree::And(
          Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Linux)),
          Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::MinGW))
        ))
      ))
    },
    ParserTestGroup {
      raw_expr: "(((windows and linux) or mingw))",
      expected_tree: Some(SystemSpecExpressionTree::Or(
        Box::new(SystemSpecExpressionTree::ParenGroup(
          Box::new(SystemSpecExpressionTree::And(
            Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows)),
            Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Linux))
          ))
        )),
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::MinGW))
      ))
    },
    ParserTestGroup {
      raw_expr: "(((windows or linux) or mingw))",
      expected_tree: Some(SystemSpecExpressionTree::Or(
        Box::new(SystemSpecExpressionTree::ParenGroup(
          Box::new(SystemSpecExpressionTree::Or(
            Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows)),
            Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Linux))
          ))
        )),
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::MinGW))
      ))
    },
    // TODO: The current parse tree currently isn't reordered for precedence.
    // For now, assume that open ands/ors are parenthesized from the right.
    // For example, the below parses as ((windows and (linux or mingw))) right now.
    // This is okay for the time being, but should be changed as it could be misleading.
    // ParserTestGroup {
    //   raw_expr: "((windows and linux or mingw))",
    //   expected_tree: 
    // },
    ParserTestGroup {
      raw_expr: "((windows or (linux and mingw)))",
      expected_tree: Some(SystemSpecExpressionTree::Or(
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows)),
        Box::new(SystemSpecExpressionTree::ParenGroup(
          Box::new(SystemSpecExpressionTree::And(
            Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Linux)),
            Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::MinGW))
          ))
        ))
      ))
    },
    ParserTestGroup {
      raw_expr: "((mingw or not (windows and linux))) something is after here",
      expected_tree: Some(SystemSpecExpressionTree::Or(
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::MinGW)),
        Box::new(SystemSpecExpressionTree::Not(
          Box::new(SystemSpecExpressionTree::ParenGroup(
            Box::new(SystemSpecExpressionTree::And(
              Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows)),
              Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Linux))
            ))
          ))
        )),
      ))
    },
    ParserTestGroup {
      raw_expr: "((not mingw and windows))",
      expected_tree: Some(SystemSpecExpressionTree::And(
        Box::new(SystemSpecExpressionTree::Not(
          Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::MinGW))
        )),
        Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows))
      ))
    },
    ParserTestGroup {
      raw_expr: "((not mingw and not windows))",
      expected_tree: Some(SystemSpecExpressionTree::And(
        Box::new(SystemSpecExpressionTree::Not(
          Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::MinGW))
        )),
        Box::new(SystemSpecExpressionTree::Not(
          Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows))
        )),
      ))
    },
    ParserTestGroup {
      raw_expr: "((not mingw or not windows))",
      expected_tree: Some(SystemSpecExpressionTree::Or(
        Box::new(SystemSpecExpressionTree::Not(
          Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::MinGW))
        )),
        Box::new(SystemSpecExpressionTree::Not(
          Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows))
        )),
      ))
    },
    ParserTestGroup {
      raw_expr: "(( feature:the-feature ))",
      expected_tree: Some(SystemSpecExpressionTree::Feature { name: String::from("the-feature") })
    },
    ParserTestGroup {
      raw_expr: "(( not feature:the-feature and feature:other-feature and windows ))",
      expected_tree: Some(SystemSpecExpressionTree::And(
        Box::new(SystemSpecExpressionTree::Not(
          Box::new(SystemSpecExpressionTree::Feature { name: String::from("the-feature") })
        )),
        Box::new(SystemSpecExpressionTree::And(
          Box::new(SystemSpecExpressionTree::Feature { name: String::from("other-feature") }),
          Box::new(SystemSpecExpressionTree::Value(SingleSystemSpec::Windows))
        ))
      ))
    }
  ];

  let invalid_expressions = [
    ParserTestGroup {
      raw_expr: "((windows (and) linux))",
      // TODO: Match given error type.
      expected_tree: None
    },
    ParserTestGroup {
      raw_expr: "((windows )and) linux))",
      expected_tree: None
    },
    ParserTestGroup {
      raw_expr: "((windows and not (not) linux))",
      expected_tree: None
    },
    ParserTestGroup {
      raw_expr: "(mingw or (windows and linux)))",
      expected_tree: None
    },
    ParserTestGroup {
      raw_expr: "((mingw or nt (windows or linux))",
      expected_tree: None
    },
    ParserTestGroup {
      raw_expr: "((mingw or n<ot (windows or linux))",
      expected_tree: None
    },
    ParserTestGroup {
      raw_expr: "((mingw or not (windows onr linux)))",
      expected_tree: None
    },
    ParserTestGroup {
      raw_expr: "(( feature :something ))",
      expected_tree: None
    },
    ParserTestGroup {
      raw_expr: "(( feature: something ))",
      expected_tree: None
    },
    ParserTestGroup {
      raw_expr: "(( featre:something ))",
      expected_tree: None
    },
    ParserTestGroup {
      raw_expr: "(( feature:: )",
      expected_tree: None
    },
    ParserTestGroup {
      raw_expr: "(( feature::something )",
      expected_tree: None
    },
    ParserTestGroup {
      raw_expr: "(( feature:feature: )",
      expected_tree: None
    }
  ];

  for ParserTestGroup { raw_expr, expected_tree } in valid_expressions.iter().chain(invalid_expressions.iter()) {
    // The iterator type given to 'None' here is just a placeholder, but is necessary because the
    // function must be instantiated with a concrete iterator type.
    match (expected_tree, parse_spec_with_diagnostic(raw_expr, None)) {
      (Some(expected), Ok(Some(ParseSuccess { value, ..}))) => {
        assert!(
          expected.exactly_matches_structure(&value),
          "Failed on {}",
          value.to_string()
        )
      },
      (None, Ok(None) | Err(_)) => continue,
      (_, invalid_result) => {
        panic!(
          "Parse result from {} was incorrect.\nExpected: {} \nActual: {}",
          raw_expr,
          expected_tree
            .as_ref()
            .map_or(String::from("None"), |the_expected| format!("{}", the_expected.to_string())),
          invalid_result
            .map_or_else(
              |err_msg| err_msg.to_string(),
              |the_acutal| the_acutal.map_or(
                String::from("None"),
                |ParseSuccess { value: tree, .. }| format!("{}", tree.to_string())
              )
            )
        )
      }
    }
  }
}