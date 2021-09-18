use std::{fs::File, io::{self, Write}, path::{Path}};

use crate::project_generator::configuration::ProjectOutputType;

const CPP_EXE_MAIN: &'static str =
"#include <iostream>

int main(int argc, const char** argv) {
\tstd::cout << \"Hello World\" << std::endl;
\treturn EXIT_SUCCESS;
}}
";

const CPP_LIB_MAIN: &'static str =
"//#include \"Your lib files\"";

pub fn generate_cpp_main<T: AsRef<Path>>(file_path: T, project_output_type: &ProjectOutputType) -> io::Result<()> {
  let main_file = File::create(file_path)?;
  
  match project_output_type {
    ProjectOutputType::Executable => write!(&main_file, "{}", CPP_EXE_MAIN)?,
    ProjectOutputType::Library => write!(&main_file, "{}", CPP_LIB_MAIN)?
  }
  Ok(())
}