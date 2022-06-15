use std::{fs::File, io::{self, Write}, path::{Path}};

use crate::project_generator::configuration::CreationProjectOutputType;

use super::configuration::OutputLibType;

const C_EXE_MAIN: &'static str =
"#include <stdio.h>

int main(int argc, const char** argv) {
\tprintf(\"Hello World!\");
\treturn 0;
}
";

const C_COMPILED_LIB_MAIN: &'static str =
"// #include \"Your library files\"";

const C_HEADER_ONLY_MAIN: &'static str =
"// Write your code here and/or #include \"Your library files\"";

pub fn generate_c_main<T: AsRef<Path>>(file_path: T, project_output_type: &CreationProjectOutputType) -> io::Result<()> {
  let main_file = File::create(file_path)?;

  match project_output_type {
    CreationProjectOutputType::Executable => write!(&main_file, "{}", C_EXE_MAIN)?,
    CreationProjectOutputType::Library(lib_type) => match lib_type {
      OutputLibType::HeaderOnly => write!(&main_file, "{}", C_HEADER_ONLY_MAIN)?,
      _ => write!(&main_file, "{}", C_COMPILED_LIB_MAIN)?,
    }
  }
  
  Ok(())
}