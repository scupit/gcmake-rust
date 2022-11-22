use std::{fs::File, io::{self, Write}, path::{Path}};

use crate::{project_generator::configuration::CreationProjectOutputType, project_info::{FinalTestFramework, CompiledOutputItem}, common::{make_c_identifier, basic_configure_replace}};

use super::{configuration::OutputLibType, cpp_test_mains::test_mains};

const CPP_EXE_MAIN: &'static str =
"#include <iostream>

int main(int argc, char** argv) {
\tstd::cout << \"Hello World\\n\";
\treturn EXIT_SUCCESS;
}
";

const CPP2_EXE_MAIN: &'static str =
"main: () -> int = {
\tstd::cout << \"Hello World\\n\";
\treturn EXIT_SUCCESS;
}
";

const CPP_COMPILED_LIB_MAIN: &'static str =
"#ifndef @TARGET_NAME@_HPP
#define @TARGET_NAME@_HPP

// #include \"Your lib files\" here

#include \"@EXPORT_HEADER@\"

class @EXPORT_MACRO@ PlaceholderClass
{

};

#endif
";

const CPP_HEADER_ONLY_MAIN: &'static str =
"#ifndef @TARGET_NAME@_HPP
#define @TARGET_NAME@_HPP

// Write your code here and/or #include \"Your library files\"

class PlacholderClass
{

};

#endif
";

pub struct TestMainInitInfo<'a> {
  pub test_framework: &'a FinalTestFramework,
  pub requires_custom_main: bool
}

pub fn generate_cpp_main<'a, T: AsRef<Path>>(
  file_path: T,
  project_output_type: &CreationProjectOutputType,
  test_init_info: Option<TestMainInitInfo<'a>>,
  full_include_prefix: &str,
  target_name: &str,
  use_cpp2: bool
) -> io::Result<()> {
  let main_file = File::create(file_path)?;

  write!(
    &main_file,
    "{}",
    get_cpp_main_file_contents(
      project_output_type,
      test_init_info,
      full_include_prefix,
      target_name,
      use_cpp2
    )
  )?;
  
  Ok(())
}

fn get_cpp_main_file_contents<'a>(
  project_output_type: &CreationProjectOutputType,
  test_init_info: Option<TestMainInitInfo<'a>>,
  full_include_prefix: &str,
  target_name: &str,
  use_cpp2: bool
) -> String {
  return match test_init_info {
    Some(TestMainInitInfo { test_framework, requires_custom_main }) => match test_framework {
      FinalTestFramework::Catch2(_) => {
        if requires_custom_main {
          String::from(test_mains::CATCH2_CUSTOM_MAIN)
        }
        else {
          String::from(test_mains::CATCH2_AUTO_MAIN)
        }
      },
      FinalTestFramework::DocTest(_) => {
        if requires_custom_main {
          String::from(test_mains::DOCTEST_CUSTOM_MAIN)
        }
        else {
          String::from(test_mains::DOCTEST_AUTO_MAIN)
        }
      },
      FinalTestFramework::GoogleTest(_) => {
        if requires_custom_main {
          String::from(test_mains::GOOGLETEST_CUSTOM_MAIN)
        }
        else {
          String::from(test_mains::GOOGLETEST_AUTO_MAIN)
        }
      }
    },
    None => {
      let target_ident_upper: String = make_c_identifier(target_name).to_uppercase();

      match project_output_type {
        CreationProjectOutputType::Executable => {
          if use_cpp2 {
            String::from(CPP2_EXE_MAIN)
          }
          else {
            String::from(CPP_EXE_MAIN)
          }
        },
        CreationProjectOutputType::Library(lib_type) => match lib_type {
          OutputLibType::HeaderOnly => {
            basic_configure_replace(
              CPP_HEADER_ONLY_MAIN,
              [
                ("TARGET_NAME", target_ident_upper)
              ]
            )
          }
          _ => {
            basic_configure_replace(
              CPP_COMPILED_LIB_MAIN,
              [
                ("EXPORT_HEADER", CompiledOutputItem::export_macro_header_include_path(full_include_prefix, target_name)),
                ("EXPORT_MACRO", CompiledOutputItem::str_export_macro(target_name)),
                ("TARGET_NAME", target_ident_upper)
              ]
            )
          }
        }
      }
    }
  }
}