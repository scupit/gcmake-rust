mod data_types;
mod item_resolver;
mod logger;
mod cmakelists_writer;

use std::{fs, io, path::{Path, PathBuf}};
use data_types::raw_types::*;
use item_resolver::FinalProjectData;
use logger::exit_error_log;

use crate::cmakelists_writer::CMakeListsWriter;

fn yaml_names_from_dir(project_root: &str) -> Vec<PathBuf> {
  let cmake_data_path: PathBuf = Path::new(project_root)
    .join("cmake_data");

  return vec![
    cmake_data_path.with_extension("yaml"), // ...../cmake_data.yaml
    cmake_data_path.with_extension("yml") // ...../cmake_data.yml
  ];
}

fn create_project_data(project_root: &str) -> Result<FinalProjectData, String> {

  for possible_cmake_data_file in yaml_names_from_dir(project_root) {
    if let io::Result::Ok(cmake_data_yaml_string) = fs::read_to_string(possible_cmake_data_file) {

      return match serde_yaml::from_str::<RawProject>(&cmake_data_yaml_string) {
        Ok(serialized_project) => Ok(FinalProjectData::new(String::from(project_root), serialized_project)),
        Err(error) => Err(error.to_string())
      }
    }
  }

  return Err(format!("Unable to find a cmake_data.yaml or cmake_data.yml file in {}", project_root));
}

fn main() {
  let args: Vec<String> = std::env::args().collect();

  let project_result = match args.len() {
    1 => create_project_data("."),
    _ => create_project_data(&args[1])
  };

  match project_result {
    Ok(project_data) => {
      // println!("Project in YAML: {:?}", project_data.get_raw_project());

      println!("Project root: {:?}", project_data.get_project_root());
      // println!("Include Dir: {:?}", project_data.get_include_dir());

      match CMakeListsWriter::new(project_data) {
        Ok(cmakelists_writer) => {
          cmakelists_writer.write_cmakelists();
        },
        Err(err) => {
          println!("{:?}", err);
        }
      }
      // println!("src dir: {:?}", project_data.get_src_dir());
      // println!("src list: {:?}", project_data.src_files);

      // println!("header dir: {:?}", project_data.get_include_dir());
      // println!("header list: {:?}", project_data.include_files);

      // println!("template-impl dir: {:?}", project_data.get_template_impl_dir());
      // println!("template-impl list: {:?}", project_data.template_impl_files);
    },
    Err(message) => exit_error_log(&message)
  }
}
