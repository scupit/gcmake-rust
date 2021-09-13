use std::{fs::File, io::{self, Write}, path::{Path}};

const CPP_MAIN_CONTENT: &'static str =
"#include <iostream>

int main(int argc, const char** argv) {{
\tstd::cout << \"Hello World\" << std::endl;
\treturn EXIT_SUCCESS;
}}
";

pub fn generate_cpp_main<T: AsRef<Path>>(file_path: T) -> io::Result<()> {
  let main_file = File::create(file_path)?;
  
  write!(&main_file, "{}", CPP_MAIN_CONTENT)?;
  Ok(())
}