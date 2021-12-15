use std::{fs::{self, File}, io::{self, Write}};


fn main() {
  let file = fs::read_to_string("./supported_dependencies.yaml")
    .expect("Unable to read supported_dependencies.yaml");

  write_dependency_config_as_string(file).expect("Failed to write supported_dependencies.rs");
}

fn write_dependency_config_as_string(dep_config_string: String) -> io::Result<()> {
  let mut file_writing = File::create("./src/project_info/raw_data_in/dependencies/supported_dependencies.rs")
    .expect("Unable to write supported_dependencies.rs");

  writeln!(&file_writing, "pub const DEPENDENCIES_YAML_STRING: &'static str = r#\"")?;
  file_writing.write_all(dep_config_string.as_bytes())?;
  writeln!(&file_writing, "\"#;")?;
  Ok(())
}