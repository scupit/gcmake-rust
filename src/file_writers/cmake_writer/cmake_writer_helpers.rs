use crate::project_info::{SystemSpecifierWrapper, SystemSpecExpressionTree, SingleSystemSpec, SystemSpecFeatureType, feature_map_for_lang};

pub fn system_contstraint_conditional_expression(system_spec: &SystemSpecifierWrapper) -> String {
  match system_spec {
    SystemSpecifierWrapper::All => String::from("TRUE"),
    SystemSpecifierWrapper::Specific(spec_tree) => {
      format!("( {} )", make_inner_system_spec_conditional_expr(spec_tree))
    }
  }
}

pub fn system_constraint_generator_expression(
  system_spec: &SystemSpecifierWrapper,
  contained_str: impl AsRef<str>
) -> String {
  match system_spec {
    SystemSpecifierWrapper::All => contained_str.as_ref().to_string(),
    SystemSpecifierWrapper::Specific(spec_tree) => {
      format!(
        "$<{}:{}>",
        make_inner_system_spec_generator_expression(spec_tree, CurrentSystemSpecContext::None),
        contained_str.as_ref()
      )
    }
  }
}

pub fn project_feature_var(feature_name: &str) -> String {
  return format!(
    "${{LOCAL_TOPLEVEL_PROJECT_NAME}}_FEATURE_{}",
    feature_name
  );
}

pub fn language_feature_name(
  feature_identifier: &str,
  feature_type: SystemSpecFeatureType
) -> String {
  assert!(
    feature_type != SystemSpecFeatureType::ProjectDefined,
    "Retrieving the 'language feature name' for a feature defined by the project doesn't make sense."
  );

  return feature_map_for_lang(feature_type).unwrap().get(feature_identifier).unwrap().to_string();
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
    SystemSpecExpressionTree::Feature { name: feature_name, feature_type } => match feature_type {
      SystemSpecFeatureType::ProjectDefined => format!(
        "$<BOOL:${{{}}}>",
        project_feature_var(feature_name)
      ),
      _ => format!(
        "$<COMPILE_FEATURES:{}>",
        language_feature_name(feature_name, *feature_type)
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
        SingleSystemSpec::CUDA => "USING_NVIDIA",
        SingleSystemSpec::Clang => "USING_CLANG",
        SingleSystemSpec::MSVC => "USING_MSVC",
        SingleSystemSpec::Emscripten => "USING_EMSCRIPTEN",
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

fn make_inner_system_spec_conditional_expr(spec_tree: &SystemSpecExpressionTree) -> String {
  match spec_tree {
    SystemSpecExpressionTree::ParenGroup(group) => {
      format!(
        "( {} )",
        make_inner_system_spec_conditional_expr(group)
      )
    },
    SystemSpecExpressionTree::Not(expr) => {
      format!(
        "NOT ({})",
        make_inner_system_spec_conditional_expr(expr)
      )
    },
    SystemSpecExpressionTree::Feature { name: feature_name, feature_type } => {
      assert!(
        *feature_type == SystemSpecFeatureType::ProjectDefined,
        "Only project-defined feature checks can be used in CMake 'if' statements. This is because language feature checks are tied to targets in CMake.\nThis is the check that failed: {}",
        spec_tree.to_string()
      );

      project_feature_var(feature_name)
    },
    SystemSpecExpressionTree::Value(value) => {
      let var_name: &str = match value {
        SingleSystemSpec::Android => "TARGET_SYSTEM_IS_ANDROID",
        SingleSystemSpec::Windows => "TARGET_SYSTEM_IS_WINDOWS",
        SingleSystemSpec::Linux => "TARGET_SYSTEM_IS_LINUX",
        SingleSystemSpec::MacOS => "TARGET_SYSTEM_IS_MACOS",
        SingleSystemSpec::Unix => "TARGET_SYSTEM_IS_UNIX",
        SingleSystemSpec::MinGW => "USING_MINGW",
        SingleSystemSpec::GCC => "USING_GCC",
        SingleSystemSpec::CUDA => "USING_NVIDIA",
        SingleSystemSpec::Clang => "USING_CLANG",
        SingleSystemSpec::MSVC => "USING_MSVC",
        SingleSystemSpec::Emscripten => "USING_EMSCRIPTEN"
      };

      var_name.to_string()
    },
    SystemSpecExpressionTree::Or(left_expr, right_expr) => {
      format!(
        "( {} ) OR ( {} )",
        make_inner_system_spec_conditional_expr(left_expr),
        make_inner_system_spec_conditional_expr(right_expr)
      )
    },
    SystemSpecExpressionTree::And(left_expr, right_expr) => {
      format!(
        "( {} ) AND ( {} )",
        make_inner_system_spec_conditional_expr(left_expr),
        make_inner_system_spec_conditional_expr(right_expr)
      )
    }
  }
}