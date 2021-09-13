use std::{fs::File, io::{self, Write}, path::{Path}};

const C_MAIN_CONTENT: &'static str =
"#include <stdio.h>

int main(int argc, const char** argv) {{
\tprintf(\"Hello World!\");
\treturn 0;
}}
";

pub fn generate_c_main<T: AsRef<Path>>(file_path: T) -> io::Result<()> {
  let main_file = File::create(file_path)?;
  
  write!(&main_file, "{}", C_MAIN_CONTENT)?;
  Ok(())
}