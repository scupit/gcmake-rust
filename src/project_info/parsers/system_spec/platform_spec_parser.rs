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

use crate::project_info::parsers::general_parser::{ParseSuccess};

use super::system_spec_parser::{SystemSpecExpressionTree, parse_spec_with_diagnostic, GivenConstraintSpecParseContext};


#[derive(Clone)]
pub enum SystemSpecifierWrapper {
  All,
  Specific(SystemSpecExpressionTree)
}

impl SystemSpecifierWrapper {
  pub fn default_include_all() -> Self {
    Self::All
  }

  // pub fn is_subset_of(&self, other: &SystemSpecifierWrapper) -> bool {
  //   unimplemented!()
  //   // return self.internal_explicit_set().is_subset(&other.internal_explicit_set());
  // }

  pub fn unwrap_specific_ref(&self) -> &SystemSpecExpressionTree {
    match self {
      Self::Specific(spec_expr_tree) => spec_expr_tree,
      _ => panic!("Tried to unwrap a system spec wrapper as 'Specific', but the wrapper doesn't contain a 'Specific' expression tree.")
    }
  }

  // self and other
  pub fn intersection(&self, other: &SystemSpecifierWrapper) -> SystemSpecifierWrapper {
    match (self, other) {
      (Self::All, Self::All) => Self::default_include_all(),
      (Self::All, Self::Specific(tree)) => Self::Specific(tree.clone()),
      (Self::Specific(tree), Self::All) => Self::Specific(tree.clone()),
      (Self::Specific(l_tree), Self::Specific(r_tree)) => {
        SystemSpecifierWrapper::Specific(
          SystemSpecExpressionTree::And(Box::new(l_tree.clone()), Box::new(r_tree.clone()))
        )
      }
    }
  }

  // self or other
  pub fn union(&self, other: &SystemSpecifierWrapper) -> SystemSpecifierWrapper {
    match (self, other) {
      (Self::All, _) => Self::default_include_all(),
      (_, Self::All) => Self::default_include_all(),
      (Self::Specific(l_tree), Self::Specific(r_tree)) => {
        SystemSpecifierWrapper::Specific(
          SystemSpecExpressionTree::Or(Box::new(l_tree.clone()), Box::new(r_tree.clone()))
        )
      }
    }
  }

  // TODO: Make this more robust. Ideally it should return true if the specific expression is
  // a tautology or simplifies to ALL. However, I haven't implemented that kind of analysis yet.
  pub fn includes_all(&self) -> bool {
    match self {
      Self::All => true,
      _ => false
    }
  }
}

impl Default for SystemSpecifierWrapper {
  fn default() -> Self {
    Self::default_include_all()
  }
}

pub fn parse_leading_constraint_spec<'a>(
  some_string: &'a str,
  context: GivenConstraintSpecParseContext
) -> Result<Option<ParseSuccess<'a, SystemSpecifierWrapper>>, String> {
  match parse_spec_with_diagnostic(some_string, context) {
    Err(err_diagnostic_msg) => Err(err_diagnostic_msg),
    // Ok(None) => Ok(ParseSuccess {
    //   value: SystemSpecCombinedInfo::default_include_all(),
    //   rest: some_string
    // }),
    Ok(None) => Ok(None),
    Ok(Some(ParseSuccess { value: spec_expr_tree, rest })) => Ok(Some(ParseSuccess {
      value: SystemSpecifierWrapper::Specific(spec_expr_tree),
      rest
    }))
  }
}