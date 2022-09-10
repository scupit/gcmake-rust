use std::{fs::File, io::{self, Write}, path::{Path}};

use crate::{project_generator::configuration::CreationProjectOutputType, project_info::FinalTestFramework};

use super::{configuration::OutputLibType, cpp_test_mains::test_mains};

const CPP_EXE_MAIN: &'static str =
"#include <iostream>

int main(int argc, char** argv) {
\tstd::cout << \"Hello World\" << std::endl;
\treturn EXIT_SUCCESS;
}
";

const CPP_COMPILED_LIB_MAIN: &'static str =
"// #include \"Your lib files\"";

const CPP_HEADER_ONLY_MAIN: &'static str =
"// Write your code here and/or #include \"Your library files\"";

pub struct TestMainInitInfo<'a> {
  pub test_framework: &'a FinalTestFramework,
  pub requires_custom_main: bool
}

pub fn generate_cpp_main<'a, T: AsRef<Path>>(
  file_path: T,
  project_output_type: &CreationProjectOutputType,
  test_init_info: Option<TestMainInitInfo<'a>>
) -> io::Result<()> {
  let main_file = File::create(file_path)?;

  write!(
    &main_file,
    "{}",
    get_cpp_main_file_contents(project_output_type, test_init_info)
  )?;
  
  Ok(())
}

fn get_cpp_main_file_contents<'a>(
  project_output_type: &CreationProjectOutputType,
  test_init_info: Option<TestMainInitInfo<'a>>
) -> &'static str {
  return match test_init_info {
    Some(TestMainInitInfo { test_framework, requires_custom_main }) => match test_framework {
      FinalTestFramework::Catch2(_) => {
        if requires_custom_main {
          test_mains::CATCH2_CUSTOM_MAIN
        }
        else {
          test_mains::CATCH2_AUTO_MAIN
        }
      },
      FinalTestFramework::DocTest(_) => {
        if requires_custom_main {
          test_mains::DOCTEST_CUSTOM_MAIN
        }
        else {
          test_mains::DOCTEST_AUTO_MAIN
        }
      },
      FinalTestFramework::GoogleTest(_) => {
        if requires_custom_main {
          test_mains::GOOGLETEST_CUSTOM_MAIN
        }
        else {
          test_mains::GOOGLETEST_AUTO_MAIN
        }
      }
    },
    None => {
      match project_output_type {
        CreationProjectOutputType::Executable => CPP_EXE_MAIN,
        CreationProjectOutputType::Library(lib_type) => match lib_type {
          OutputLibType::HeaderOnly => CPP_HEADER_ONLY_MAIN,
          _ => CPP_COMPILED_LIB_MAIN
        }
      }
    }
  }
}