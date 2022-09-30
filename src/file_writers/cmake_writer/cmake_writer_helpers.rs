use crate::project_info::{SystemSpecifierWrapper, SystemSpecExpressionTree, SingleSystemSpec};


pub fn system_constraint_expression(
  system_spec: &SystemSpecifierWrapper,
  contained_str: &str
) -> String {
  match system_spec {
    SystemSpecifierWrapper::All => contained_str.to_string(),
    SystemSpecifierWrapper::Specific(spec_tree) => {
      format!(
        "$<{}:{}>",
        make_inner_system_spec_generator_expression(spec_tree, CurrentSystemSpecContext::None),
        contained_str
      )
    }
  }
}

enum CurrentSystemSpecContext {
  None,
  And,
  Or
}

fn make_inner_system_spec_generator_expression(
  spec_tree: &SystemSpecExpressionTree,
  context: CurrentSystemSpecContext
) -> String {
  match spec_tree {
    SystemSpecExpressionTree::ParenGroup(group) => make_inner_system_spec_generator_expression(group, context),
    SystemSpecExpressionTree::Not(expr) => {
      format!(
        "$<NOT:{}>",
        make_inner_system_spec_generator_expression(expr, CurrentSystemSpecContext::None)
      )
    },
    SystemSpecExpressionTree::Value(value) => {
      // These variables are all defined in the gcmake-variables.cmake util file.
      let var_str: &str = match value {
        SingleSystemSpec::Android => "TARGET_SYSTEM_IS_ANDROID",
        SingleSystemSpec::Windows => "TARGET_SYSTEM_IS_WINDOWS",
        SingleSystemSpec::Linux => "TARGET_SYSTEM_IS_LINUX",
        SingleSystemSpec::MacOS => "TARGET_SYSTEM_IS_MACOS",
        SingleSystemSpec::Unix => "TARGET_SYSTEM_IS_UNIX",
        SingleSystemSpec::MinGW => "USING_MINGW",
        SingleSystemSpec::GCC => "USING_GCC",
        SingleSystemSpec::Clang => "USING_CLANG",
        SingleSystemSpec::MSVC => "USING_MSVC",
        SingleSystemSpec::Emscripten => "USING_MSVC",
      };

      format!("$<BOOL:${{{}}}>", var_str)
    },
    SystemSpecExpressionTree::And(left_expr, right_expr) => {
      let left_string = make_inner_system_spec_generator_expression(left_expr, CurrentSystemSpecContext::And);
      let right_string = make_inner_system_spec_generator_expression(right_expr, CurrentSystemSpecContext::And);

      if let CurrentSystemSpecContext::And = context {
        format!(
          "{},{}",
          left_string,
          right_string
        )
      }
      else {
        format!(
          "$<AND:{},{}>",
          left_string,
          right_string
        )
      }
    },
    SystemSpecExpressionTree::Or(left_expr, right_expr) => {
      let left_string = make_inner_system_spec_generator_expression(left_expr, CurrentSystemSpecContext::Or);
      let right_string = make_inner_system_spec_generator_expression(right_expr, CurrentSystemSpecContext::Or);

      if let CurrentSystemSpecContext::Or = context {
        format!(
          "{},{}",
          left_string,
          right_string
        )
      }
      else {
        format!(
          "$<OR:{},{}>",
          left_string,
          right_string
        )
      }
    }
  }
}