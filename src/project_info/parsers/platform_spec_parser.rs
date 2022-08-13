/*
  System specifier
  ------------------------------

  "Systems" in this scenario mean the target operating system, current specialized compiler 'environment'
  (such as MinGW), and (TODO) target architecture.

  - android
  - windows
  - linux
  - macos
  - mingw

  Only include on the given "systems":
  - ((macos))
  - ((windows and linux))

  Omit from all the given "systems":
  - ((omit windows and linux))
*/

use std::{collections::HashSet, iter::FromIterator, fmt::Debug};

use regex::Regex;

use enum_iterator::Sequence;

const ANDROID_SPEC_STR: &'static str = "android";
const WINDOWS_SPEC_STR: &'static str = "windows";
const LINUX_SPEC_STR: &'static str = "linux";
const MACOS_SPEC_STR: &'static str = "macos";
const MINGW_SPEC_STR: &'static str = "mingw";
const UNIX_SPEC_STR: &'static str = "unix";

#[derive(Clone)]
pub enum SystemSpecMode {
  Include,
  Omit
}

#[derive(Hash, PartialEq, Eq, Sequence, Clone)]
pub enum SingleSystemSpec {
  Android,
  Windows,
  Linux,
  MacOS,
  MinGW,
  Unix
}

impl SingleSystemSpec {
  fn is_valid_spec_str(spec_str: &str) -> bool {
    return Self::get_from_str(spec_str).is_some();
  }

  fn get_from_str(spec_str: &str) -> Option<Self> {
    return match spec_str {
      ANDROID_SPEC_STR => Some(Self::Android),
      WINDOWS_SPEC_STR => Some(Self::Windows),
      LINUX_SPEC_STR => Some(Self::Linux),
      MACOS_SPEC_STR => Some(Self::MacOS),
      MINGW_SPEC_STR => Some(Self::MinGW),
      UNIX_SPEC_STR => Some(Self::Unix),
      _ => None
    }
  }
}

impl Debug for SingleSystemSpec {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let the_str: &str = match self {
      Self::Android => ANDROID_SPEC_STR,
      Self::Windows => WINDOWS_SPEC_STR,
      Self::Linux => LINUX_SPEC_STR,
      Self::MacOS => MACOS_SPEC_STR,
      Self::MinGW => MINGW_SPEC_STR,
      Self::Unix => UNIX_SPEC_STR,
    };

    write!(f, "{}", the_str)
  }
}

#[derive(Clone)]
pub enum SpecSetWrapper {
  All,
  Specific(HashSet<SingleSystemSpec>)
}

impl SpecSetWrapper {
  fn from_set(spec_set: HashSet<SingleSystemSpec>) -> Self {
    let all_possible_specs: HashSet<SingleSystemSpec> = enum_iterator::all::<SingleSystemSpec>()
      .collect();

    return if all_possible_specs.difference(&spec_set).collect::<Vec<_>>().is_empty()
      { SpecSetWrapper::All }
      else { SpecSetWrapper::Specific(spec_set) };
  }
}

#[derive(Clone)]
pub struct SystemSpecCombinedInfo {
  mode: SystemSpecMode,
  spec_set: SpecSetWrapper
}

impl SystemSpecCombinedInfo {
  pub fn default_include_all() -> Self {
    Self {
      mode: SystemSpecMode::Include,
      spec_set: SpecSetWrapper::All
    }
  }

  fn default_omit_all() -> Self {
    Self {
      mode: SystemSpecMode::Omit,
      spec_set: SpecSetWrapper::All
    }
  }

  pub fn get_mode(&self) -> &SystemSpecMode {
    &self.mode
  }

  pub fn is_subset_of(&self, other: &SystemSpecCombinedInfo) -> bool {
    return self.internal_explicit_set().is_subset(&other.internal_explicit_set());
  }

  pub fn intersection(&self, other: &SystemSpecCombinedInfo) -> SystemSpecCombinedInfo {
    if self.includes_all() && other.includes_all() {
      return Self::default_include_all();
    }

    let new_spec_set: HashSet<SingleSystemSpec> = self.internal_explicit_set()
      .intersection(&other.internal_explicit_set())
      .map(|single_spec_ref| single_spec_ref.clone())
      .collect();

    return if new_spec_set.is_empty() {
      Self::default_omit_all()
    }
    else {
      SystemSpecCombinedInfo {
        mode: SystemSpecMode::Include,
        spec_set: SpecSetWrapper::Specific(new_spec_set)
      }
    }
  }

  pub fn union(&self, other: &SystemSpecCombinedInfo) -> SystemSpecCombinedInfo {
    if self.includes_all() && other.includes_all() {
      return Self::default_include_all();
    }

    let merged_set: HashSet<SingleSystemSpec> = self.internal_explicit_set()
      .union(&other.internal_explicit_set())
      .map(|single_spec_ref| single_spec_ref.clone())
      .collect();
    
    let merged_set_contains_all: bool = merged_set.is_superset(
      &enum_iterator::all::<SingleSystemSpec>().collect()
    );

    return if merged_set_contains_all {
      Self::default_include_all()
    }
    else {
      SystemSpecCombinedInfo {
        mode: SystemSpecMode::Include,
        spec_set: SpecSetWrapper::Specific(merged_set)
      }
    }
  }

  pub fn omitted_specs(&self) -> Option<HashSet<SingleSystemSpec>> {
    match &self.mode {
      SystemSpecMode::Include => None,
      SystemSpecMode::Omit => match &self.spec_set {
        SpecSetWrapper::All => Some(HashSet::from_iter(enum_iterator::all::<SingleSystemSpec>())),
        SpecSetWrapper::Specific(spec_set) => Some(spec_set.clone())
      }
    }
  }

  pub fn included_specs(&self) -> Option<HashSet<SingleSystemSpec>> {
    match &self.mode {
      SystemSpecMode::Omit => None,
      SystemSpecMode::Include => match &self.spec_set {
        SpecSetWrapper::All => Some(HashSet::from_iter(enum_iterator::all::<SingleSystemSpec>())),
        SpecSetWrapper::Specific(spec_set) => Some(spec_set.clone())
      }
    }
  }

  pub fn explicit_name_list(&self) -> Vec<&str> {
    return self.internal_explicit_set()
      .iter()
      .map(|spec| match spec {
        SingleSystemSpec::Android => ANDROID_SPEC_STR,
        SingleSystemSpec::Linux => LINUX_SPEC_STR,
        SingleSystemSpec::MinGW => MINGW_SPEC_STR,
        SingleSystemSpec::Windows => WINDOWS_SPEC_STR,
        SingleSystemSpec::MacOS => MACOS_SPEC_STR,
        SingleSystemSpec::Unix => UNIX_SPEC_STR
      })
      .collect();
  }

  pub fn includes_all(&self) -> bool {
    match (&self.mode, &self.spec_set) {
      (SystemSpecMode::Include, SpecSetWrapper::All) => true,
      _ => false
    }
  }

  pub fn omits_all(&self) -> bool {
    match (&self.mode, &self.spec_set) {
      (SystemSpecMode::Omit, SpecSetWrapper::All) => true,
      _ => false
    }
  }

  fn internal_explicit_set(&self) -> HashSet<SingleSystemSpec> {
    match &self.spec_set {
      SpecSetWrapper::All => match &self.mode {
        SystemSpecMode::Include => enum_iterator::all::<SingleSystemSpec>().collect(),
        SystemSpecMode::Omit => HashSet::new()
      },
      SpecSetWrapper::Specific(given_set) => match &self.mode {
        SystemSpecMode::Include => given_set.clone(),
        SystemSpecMode::Omit => enum_iterator::all::<SingleSystemSpec>()
          .collect::<HashSet<SingleSystemSpec>>()
          .difference(given_set)
          .map(|single_spec_borrow| single_spec_borrow.clone())
          .collect()
      }
    }
  }
}

impl Default for SystemSpecCombinedInfo {
  fn default() -> Self {
    Self::default_include_all()
  }
}

pub type SystemSpecParseSuccessData<'a> = (Option<SystemSpecCombinedInfo>, &'a str);

pub type SystemSpecParseResult<'a> = Result<SystemSpecParseSuccessData<'a>, String>;

pub fn parse_leading_system_spec<'a>(full_str: &'a str) -> SystemSpecParseResult<'a> {
  /*
    Capture groups
    ------------------------------
    
    0: The entire captured str
    1: The entire specifier including double parentheses, if it's found.
    2: 'omit', if it was specified and isn't the only thing in parentheses.
    3: The list of specifiers. Not yet validated, but guaranteed to only contain
        letters, numbers, and spaces.
  */
  let spec_regex = Regex::new("^(\\(\\((omit)?([a-zA-Z0-9 ]+)?\\)\\))")
    .unwrap();

  let trimmed_full_str: &'a str = full_str.trim();

  match spec_regex.captures(trimmed_full_str) {
    None => return Ok((None, trimmed_full_str)),
    Some(all_captures) => {
      let full_captured_specifier: &str = all_captures.get(1).unwrap().as_str();
      let has_omit: bool = all_captures.get(2).is_some();
      let mut spec_set: HashSet<SingleSystemSpec> = HashSet::new();

      let unvalidated_spec_str_list = all_captures.get(3).unwrap()
        .as_str()
        .split(' ')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

      for spec_str in unvalidated_spec_str_list {
        if spec_str == "and" {
          continue;
        }

        match SingleSystemSpec::get_from_str(spec_str) {
          Some(valid_spec) => {
            spec_set.insert(valid_spec);
          },
          None => return Err(format!(
            "Invalid system specifier '{}' given in '{}'",
            spec_str,
            full_captured_specifier
          ))
        }
      }

      if spec_set.is_empty() {
        return Err(format!(
          "Given full system specifier '{}' does not specify any single systems, and would have no effect in its current form. System specifiers must specify at least one single system/environment. Ex: ((omit windows and linux))",
          full_captured_specifier
        ))
      }

      let full_system_spec_info: SystemSpecCombinedInfo = SystemSpecCombinedInfo {
        spec_set: SpecSetWrapper::from_set(spec_set),
        mode: if has_omit
          { SystemSpecMode::Omit }
          else { SystemSpecMode::Include }
      };

      match (&full_system_spec_info.mode, &full_system_spec_info.spec_set) {
        (SystemSpecMode::Omit, SpecSetWrapper::All) => return Err(format!(
          "Given full system specifier '{}' omits all platforms and systems. If this is intentional, just remove the entry entirely.",
          full_captured_specifier
        )),
        (_, SpecSetWrapper::Specific(used_specs)) => {
          assert!(
            !used_specs.is_empty(),
            "At this point, the spec set should always contain at least one value."
          )
        }
        _ => ()
      }
      
      return Ok((
        Some(full_system_spec_info),
        trimmed_full_str.strip_prefix(full_captured_specifier).unwrap()
      ))
    }
  }

}