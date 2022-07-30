// TODO: Write tests. This is an easy module to unit test.
use regex::Regex;

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

// A successful parse means that all namespaces and target strings are properly formatted
// and that the LinkSpecifier contains at least one namespace and at least one target name.
#[derive(Clone)]
pub struct LinkSpecifier {
  original_specifier_string: String,
  namespace_stack: Vec<String>,
  target_list: Vec<String>,
  access_mode: LinkAccessMode
}

impl LinkSpecifier {
  pub fn get_spec_string(&self) -> &str {
    &self.original_specifier_string
  }

  pub fn get_access_mode(&self) -> &LinkAccessMode {
    &self.access_mode
  }

  pub fn get_target_list(&self) -> &Vec<String> {
    &self.target_list
  }

  pub fn get_namespace_stack(&self) -> &Vec<String> {
    &self.namespace_stack
  }

  pub fn parse_from(
    link_spec: impl AsRef<str>,
    access_mode: LinkAccessMode
  ) -> Result<Self, String> {
    let specifier_string: String = link_spec.as_ref().to_string();
    let (
      open_brace_indices,
      close_brace_indices
    ) = brace_indices(&specifier_string);

    if open_brace_indices.len() > 1 {
      return Self::parsing_error(&specifier_string, "Too many opening braces. There should be 1 or 0 opening braces.");
    }
    else if close_brace_indices.len() > 1 {
      return Self::parsing_error(&specifier_string, "Too many closing braces. There should be 1 or 0 closing braces.");
    }
    else if open_brace_indices.len() != close_brace_indices.len() {
      return Self::parsing_error(&specifier_string, "Unequal number of opening and closing braces.");
    }

    if open_brace_indices.len() == 1 {
      assert!(
        close_brace_indices.len() == 1,
        "When there is one opening brace and brace count is valid, there should be only one closing brace."
      );
      let open_brace_index: usize = open_brace_indices[0];
      let close_brace_index: usize = close_brace_indices[0];

      if open_brace_index > close_brace_index {
        return Self::parsing_error(&specifier_string, "Opening brace must be before the closing brace.");
      }
      else if close_brace_index != specifier_string.trim_end().len() - 1 {
        return Self::parsing_error(&specifier_string, "Closing brace must be the last non-whitespace character in a link specifier string.");
      }

      let target_list: Vec<String> = match Self::parse_target_list(&specifier_string[open_brace_index + 1..close_brace_index]) {
        Ok(the_list) => the_list,
        Err(err) => return Self::parsing_error(&specifier_string, err.to_string())
      };

      if target_list.is_empty() {
        return Self::parsing_error(&specifier_string, "At least one target must be provided.");
      }

      let namespace_stack: Vec<String> = match Self::parse_namespace_list(&specifier_string[..open_brace_index], true) {
        Ok(the_stack) => the_stack,
        Err(err) => return Self::parsing_error(&specifier_string, err.to_string())
      };

      return Ok(Self {
        original_specifier_string: specifier_string,
        namespace_stack,
        target_list,
        access_mode
      });
    }
    else {
      assert!(
        open_brace_indices.is_empty() && close_brace_indices.is_empty(),
        "There are no opening or closing braces"
      );
      
      let mut namespace_stack: Vec<String> = match Self::parse_namespace_list(&specifier_string, false) {
        Ok(the_stack) => the_stack,
        Err(err) => return Self::parsing_error(&specifier_string, err.to_string())
      };

      assert!(
        !namespace_stack.is_empty(),
        "Namespace stack should not be empty after parsing"
      );

      if namespace_stack.len() == 1 {
        return Self::parsing_error(
          &specifier_string,
          format!(
            "Only the target name \"{}\" was given, but it's missing a namespace. Try namespacing the target name. Ex: \"some_project_name::{}\"",
            namespace_stack[0],
            namespace_stack[0]
          )
        )
      }
      
      let target_list: Vec<String> = vec![namespace_stack.last().unwrap().to_string()];
      namespace_stack.pop();

      return Ok(Self {
        original_specifier_string: specifier_string,
        namespace_stack,
        target_list,
        access_mode
      })
      
    }
  }

  fn parse_target_list(target_list_str: &str) -> Result<Vec<String>, String> {
    let mut verified_target_names: Vec<String> = Vec::new();

    for untrimmed_target_name in TARGET_LIST_SEPARATOR_REGEX.split(target_list_str) {
      let target_name: &str = untrimmed_target_name.trim();

      if VALID_SINGLE_ITEM_SPEC_REGEX.is_match(target_name) {
        verified_target_names.push(target_name.to_string());
      }
      else {
        return Err(format!("Invalid target specifier \"{}\".", target_name));
      }
    }

    return Ok(verified_target_names);
  }

  // namespace_list_str must include the final separator (::) when 
  fn parse_namespace_list(
    namespace_list_str: &str,
    was_braced_target_list_already_parsed: bool
  ) -> Result<Vec<String>, String> {
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

    let mut valid_split_results: Vec<String> = Vec::new();

    for raw_namespace_string in raw_split_results {
      if raw_namespace_string.is_empty() {
        return Err(format!(
          "Namespaces and/or target names cannot be empty"
        ))
      }
      else if VALID_SINGLE_ITEM_SPEC_REGEX.is_match(&raw_namespace_string) {
        valid_split_results.push(raw_namespace_string.to_string());
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
      "Error when parsing link specifier \"{}\": {}",
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