use std::{fs::File, io::{self, Write}, path::{Path}};

use crate::{project_generator::configuration::CreationProjectOutputType, project_info::CompiledOutputItem, common::{make_c_identifier, basic_configure_replace}};

use super::configuration::OutputLibType;

const C_EXE_MAIN: &'static str =
"#include <stdio.h>

int main(int argc, char** argv) {
\tprintf(\"Hello World!\");
\treturn 0;
}
";

const C_COMPILED_LIB_MAIN: &'static str =
"#ifndef @TARGET_NAME@_H
#define @TARGET_NAME@_H

// #include \"Your library files\"

#include \"@EXPORT_HEADER@\"

inline int @EXPORT_MACRO@ placholderFunc(void) {
  return 2;
}
";

const C_HEADER_ONLY_MAIN: &'static str =
"#ifndef @TARGET_NAME@_H
#define @TARGET_NAME@_H

// Write your code here and/or #include \"Your library files\"

inline int placeholderFunc(void) {
  return 2;
}

#endif
";

pub fn generate_c_main<T: AsRef<Path>>(
  file_path: T,
  project_output_type: &CreationProjectOutputType,
  full_include_prefix: &str,
  target_name: &str
) -> io::Result<()> {
  let main_file = File::create(file_path)?;

  let target_ident_upper: String = make_c_identifier(target_name).to_uppercase();

  match project_output_type {
    CreationProjectOutputType::Executable => write!(&main_file, "{}", C_EXE_MAIN)?,
    CreationProjectOutputType::Library(lib_type) => match lib_type {
      OutputLibType::HeaderOnly => {
        let resolved_main_content: String = basic_configure_replace(
          C_HEADER_ONLY_MAIN,
          [
            ("TARGET_NAME", target_ident_upper)
          ]
        );

        write!(&main_file, "{}", resolved_main_content)?
      },
      _ => {
        let resolved_main_content: String = basic_configure_replace(
          C_COMPILED_LIB_MAIN,
          [
            ("EXPORT_MACRO", CompiledOutputItem::str_export_macro(target_name)),
            ("EXPORT_HEADER", CompiledOutputItem::export_macro_header_include_path(full_include_prefix, target_name)),
            ("TARGET_NAME", target_ident_upper)
          ]
        );

        write!(&main_file, "{}", resolved_main_content)?
      },
    }
  }
  
  Ok(())
}