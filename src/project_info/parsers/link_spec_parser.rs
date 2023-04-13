// TODO: Write tests. This is an easy module to unit test.
use regex::Regex;

use crate::project_info::GivenConstraintSpecParseContext;

use super::{system_spec::platform_spec_parser::{SystemSpecifierWrapper, parse_leading_constraint_spec}, general_parser::ParseSuccess};

use std::{hash::{Hash, Hasher}, collections::VecDeque};

const NAMESPACE_SEPARATOR: &'static str = "::";

lazy_static! {
  static ref TARGET_LIST_SEPARATOR_REGEX: Regex = new_regex_or_panic(" *, *");
  static ref VALID_SINGLE_ITEM_SPEC_REGEX: Regex = new_regex_or_panic("^[-_a-zA-Z0-9]+$");
}

fn new_regex_or_panic(regex_str: &str) -> Regex {
  return match Regex::new(regex_str) {
    Ok(r) => r,
    Err(failure_error) => panic!("{}", failure_error)
  }
}

#[derive(PartialEq, PartialOrd, Clone)]
pub enum LinkAccessMode {
  UserFacing,
  Internal
}

impl LinkAccessMode {
  pub fn satisfies(&self, privilege: &LinkAccessMode) -> bool {
    return self >= privilege;
  }
}

#[derive(Clone)]
pub struct LinkSpecifierTarget {
  name: String,
  system_spec_info: SystemSpecifierWrapper
}

impl LinkSpecifierTarget {
  pub fn get_name(&self) -> &str {
    &self.name
  }

  pub fn get_system_spec_info(&self) -> &SystemSpecifierWrapper {
    &self.system_spec_info
  }
}

impl Hash for LinkSpecifierTarget {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name.hash(state);
  }
}

pub type LinkSpecTargetList = Vec<LinkSpecifierTarget>;

/*
  A successful parse means that all namespaces and target strings are properly formatted
  and that the LinkSpecifier contains at least one namespace and at least one target name.

  System specifiers can be placed before both the whole link specifier, and before
  each individual specifier in a list.

  - ((windows)) SFML::main
  - SFML::{ ((windows)) main }

  However, this can cause conflicting specifier issues in these situations:

  - ((omit windows and mingw)) wxWidgets::{ core, ((mingw)) base }

  - ((omit mingw)) wxWidgets::base
  - ((mingw)) wxWidgets::base
*/
#[derive(Clone)]
pub struct LinkSpecifier {
  original_specifier_string: String,
  namespace_queue: VecDeque<String>,
  // target_list: Vec<String>,
  target_list: LinkSpecTargetList,
  access_mode: LinkAccessMode
}

impl LinkSpecifier {
  pub fn join_some_namespace_queue(namespace_queue: &VecDeque<String>) -> String {
    return namespace_queue.iter()
      .map(|the_str| &the_str[..])
      .collect::<Vec<&str>>()
      .join("::");
  }

  pub fn original_spec_str(&self) -> &str {
    &self.original_specifier_string
  }

  pub fn get_access_mode(&self) -> &LinkAccessMode {
    &self.access_mode
  }

  pub fn get_target_list(&self) -> &LinkSpecTargetList {
    &self.target_list
  }

  pub fn get_namespace_queue(&self) -> &VecDeque<String> {
    &self.namespace_queue
  }

  pub fn parse_with_full_permissions(
    link_spec: impl AsRef<str>,
    maybe_valid_feature_list: Option<&Vec<&str>>
  ) -> Result<Self, String> {
    Self::parse_from(link_spec, LinkAccessMode::Internal, maybe_valid_feature_list)
  }

  pub fn parse_from(
    link_spec: impl AsRef<str>,
    access_mode: LinkAccessMode,
    maybe_valid_feature_list: Option<&Vec<&str>>
  ) -> Result<Self, String> {
    let full_specifier_string: String = link_spec.as_ref().to_string();

    let maybe_system_spec: Option<SystemSpecifierWrapper>;
    let specifiers_only_str: &str;

    {
      let parsing_context = GivenConstraintSpecParseContext {
        is_before_output_name: false,
        maybe_valid_feature_list
      };

      match parse_leading_constraint_spec(&full_specifier_string, parsing_context)? {
        Some(ParseSuccess { value, rest }) => {
          maybe_system_spec = Some(value);
          specifiers_only_str = rest;
        },
        None => {
          maybe_system_spec = None;
          specifiers_only_str = &full_specifier_string;
        }
      }
    }

    let (
      open_brace_indices,
      close_brace_indices
    ) = brace_indices(&specifiers_only_str);

    if open_brace_indices.len() > 1 {
      return Self::parsing_error(&full_specifier_string, "Too many opening braces. There should be 1 or 0 opening braces.");
    }
    else if close_brace_indices.len() > 1 {
      return Self::parsing_error(&full_specifier_string, "Too many closing braces. There should be 1 or 0 closing braces.");
    }
    else if open_brace_indices.len() != close_brace_indices.len() {
      return Self::parsing_error(&full_specifier_string, "Unequal number of opening and closing braces.");
    }

    if open_brace_indices.len() == 1 {
      assert!(
        close_brace_indices.len() == 1,
        "When there is one opening brace and brace count is valid, there should be only one closing brace."
      );
      let open_brace_index: usize = open_brace_indices[0];
      let close_brace_index: usize = close_brace_indices[0];

      if open_brace_index > close_brace_index {
        return Self::parsing_error(&full_specifier_string, "Opening brace must be before the closing brace.");
      }
      else if close_brace_index != specifiers_only_str.trim_end().len() - 1 {
        return Self::parsing_error(&full_specifier_string, "Closing brace must be the last non-whitespace character in a link specifier string.");
      }

      let target_list_parse_result = Self::parse_target_list(
        &specifiers_only_str[open_brace_index + 1..close_brace_index],
        &maybe_system_spec,
        maybe_valid_feature_list
      );

      let target_list: LinkSpecTargetList = match target_list_parse_result {
        Ok(the_list) => the_list,
        Err(err) => return Self::parsing_error(&full_specifier_string, err.to_string())
      };

      if target_list.is_empty() {
        return Self::parsing_error(&full_specifier_string, "At least one target must be provided.");
      }

      let namespace_queue: VecDeque<String> = match Self::parse_namespace_list(&specifiers_only_str[..open_brace_index], true) {
        Ok(the_stack) => the_stack,
        Err(err) => return Self::parsing_error(&full_specifier_string, err.to_string())
      };

      return Ok(Self {
        original_specifier_string: full_specifier_string,
        namespace_queue,
        target_list,
        access_mode
      });
    }
    else {
      assert!(
        open_brace_indices.is_empty() && close_brace_indices.is_empty(),
        "There are no opening or closing braces"
      );
      
      let mut namespace_queue: VecDeque<String> = match Self::parse_namespace_list(&specifiers_only_str, false) {
        Ok(the_stack) => the_stack,
        Err(err) => return Self::parsing_error(&full_specifier_string, err.to_string())
      };

      assert!(
        !namespace_queue.is_empty(),
        "Namespace stack should not be empty after parsing"
      );

      if namespace_queue.len() == 1 {
        return Self::parsing_error(
          &full_specifier_string,
          format!(
            "Only the target name \"{}\" was given, but it's missing a namespace. Try namespacing the target name. Ex: \"some_project_name::{}\"",
            namespace_queue[0],
            namespace_queue[0]
          )
        )
      }
      
      let target_list: LinkSpecTargetList = vec![LinkSpecifierTarget {
        name: namespace_queue.pop_back().unwrap().to_string(),
        system_spec_info: maybe_system_spec.unwrap_or_default()
      }];

      return Ok(Self {
        original_specifier_string: full_specifier_string,
        namespace_queue,
        target_list,
        access_mode
      })
    }
  }

  fn parse_target_list(
    target_list_str: &str,
    full_link_set_system_spec: &Option<SystemSpecifierWrapper>,
    maybe_valid_feature_list: Option<&Vec<&str>>
  ) -> Result<LinkSpecTargetList, String> {
    let mut verified_targets: LinkSpecTargetList = Vec::new();

    for full_target_spec in TARGET_LIST_SEPARATOR_REGEX.split(target_list_str) {
      let target_specific_system_spec: Option<SystemSpecifierWrapper>;
      let untrimmed_target_name: &str;

      {
        let parsing_context = GivenConstraintSpecParseContext {
          is_before_output_name: false,
          maybe_valid_feature_list
        };

        match parse_leading_constraint_spec(full_target_spec, parsing_context)? {
          Some(ParseSuccess { value, rest }) => {
            target_specific_system_spec = Some(value);
            untrimmed_target_name = rest;
          },
          None => {
            target_specific_system_spec = None;
            untrimmed_target_name = full_target_spec;
          }
        }
      }

      let target_name: &str = untrimmed_target_name.trim();

      if VALID_SINGLE_ITEM_SPEC_REGEX.is_match(target_name) {

        if full_link_set_system_spec.is_some() && !target_specific_system_spec.is_some() {
          return Err(format!(
            "When a link set is prefixed with a system specifier, targets in the link set cannot be individually prefixed with system specifiers. However, the target '{}' is prefixed with a system specifier.\n\tSee here -> '{}'",
            target_name,
            full_target_spec
          ));
        }

        verified_targets.push(LinkSpecifierTarget {
          name: target_name.to_string(),
          system_spec_info: target_specific_system_spec.unwrap_or(
            full_link_set_system_spec.clone()
              .unwrap_or_default()
          )
        });
      }
      else {
        return Err(format!("Invalid target specifier \"{}\".", target_name));
      }
    }

    return Ok(verified_targets);
  }

  // namespace_list_str must include the final separator (::) when 
  fn parse_namespace_list(
    namespace_list_str: &str,
    was_braced_target_list_already_parsed: bool
  ) -> Result<VecDeque<String>, String> {
    let mut raw_split_results: Vec<&str> = namespace_list_str.split(NAMESPACE_SEPARATOR)
      .map(|split_result| split_result.trim())
      .collect();

    if was_braced_target_list_already_parsed {
      assert!(
        raw_split_results.last().unwrap().trim().is_empty(),
        "There should be an 'empty' namespace section after parsing the braced target list."
      );
    
      raw_split_results.pop();
    }

    let mut valid_split_results: VecDeque<String> = VecDeque::new();

    for raw_namespace_string in raw_split_results {
      if raw_namespace_string.is_empty() {
        return Err(format!(
          "Namespaces and/or target names cannot be empty"
        ))
      }
      else if VALID_SINGLE_ITEM_SPEC_REGEX.is_match(&raw_namespace_string) {
        valid_split_results.push_back(raw_namespace_string.to_string());
      }
      else {
        return Err(format!(
          "Invalid value '{}'",
          raw_namespace_string
        ));
      }
    }

    return Ok(valid_split_results);
  }

  fn parsing_error(
    spec_str: &str,
    error_msg: impl AsRef<str>
  ) -> Result<Self, String> {
    return Err(format!(
      "Error when parsing link specifier \"{}\":\n\t{}",
      spec_str,
      error_msg.as_ref()
    ));
  }
}

fn brace_indices(some_str: &str) -> (Vec<usize>, Vec<usize>) {
  let mut open_bracket_indices: Vec<usize> = Vec::new();
  let mut close_bracket_indices: Vec<usize> = Vec::new();

  let mut search_slice: &str = some_str;

  while let Some(found_index) = search_slice.rfind('{') {
    search_slice = &search_slice[..found_index];
    open_bracket_indices.push(found_index);
  }

  search_slice = some_str;

  while let Some(found_index) = search_slice.rfind('}') {
    search_slice = &search_slice[..found_index];
    close_bracket_indices.push(found_index);
  }

  return (
    open_bracket_indices,
    close_bracket_indices
  );
}