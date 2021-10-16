use std::{fs::File, io::{self, Write}, path::{Path}};

use crate::project_generator::configuration::ProjectOutputType;

const C_EXE_MAIN: &'static str =
"#include <stdio.h>

int main(int argc, const char** argv) {
\tprintf(\"Hello World!\");
\treturn 0;
}
";

const C_LIB_MAIN: &'static str =
"//#include \"Your library files\"";

pub fn generate_c_main<T: AsRef<Path>>(file_path: T, project_output_type: &ProjectOutputType) -> io::Result<()> {
  let main_file = File::create(file_path)?;

  match project_output_type {
    ProjectOutputType::Executable => write!(&main_file, "{}", C_EXE_MAIN)?,
    ProjectOutputType::Library(_) => write!(&main_file, "{}", C_LIB_MAIN)?,
  }
  
  Ok(())
}