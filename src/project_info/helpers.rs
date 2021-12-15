use std::{path::{PathBuf, Path}, fs, io};

use regex::{Captures, Regex};

use super::{raw_data_in::{RawProject, RawSubproject, ProjectLike}, path_manipulation::cleaned_pathbuf, final_project_configurables::LinkInfo};

pub fn yaml_names_from_dir(project_root: &str) -> Vec<PathBuf> {
  let cmake_data_path: PathBuf = Path::new(project_root)
    .join("cmake_data");

  return vec![
    cmake_data_path.with_extension("yaml"), // ...../cmake_data.yaml
    cmake_data_path.with_extension("yml") // ...../cmake_data.yml
  ];
}

pub fn create_project_data(project_root: &str) -> Result<RawProject, String> {
  for possible_cmake_data_file in yaml_names_from_dir(project_root) {
    if let io::Result::Ok(cmake_data_yaml_string) = fs::read_to_string(possible_cmake_data_file) {

      return match serde_yaml::from_str::<RawProject>(&cmake_data_yaml_string) {
        Ok(serialized_project) => Ok(serialized_project),
        Err(error) => Err(error.to_string())
      }
    }
  }

  return Err(format!("Unable to find a cmake_data.yaml or cmake_data.yml file in {}", project_root));
}

pub fn create_subproject_data(project_root: &str) -> Result<RawSubproject, String> {
  for possible_cmake_data_file in yaml_names_from_dir(project_root) {
    if let io::Result::Ok(cmake_data_yaml_string) = fs::read_to_string(possible_cmake_data_file) {

      return match serde_yaml::from_str::<RawSubproject>(&cmake_data_yaml_string) {
        Ok(serialized_project) => Ok(serialized_project),
        Err(error) => Err(error.to_string())
      }
    }
  }

  return Err(format!("Unable to find a cmake_data.yaml or cmake_data.yml file in {}", project_root));
}

pub fn populate_files(dir: &Path, file_list: &mut Vec<PathBuf>) -> io::Result<()> {
  if dir.is_dir() {
    for dirent in fs::read_dir(dir)? {
      let path = dirent?.path();
      if path.is_dir() {
        populate_files(&path, file_list)?;
      }
      else {
        file_list.push(cleaned_pathbuf(path));
      }
    }
  }
  Ok(())
}

// Note that index 0 containes the whole capture (string matching)
pub fn extract_capture_str<'a>(captures: &'a Captures, index: usize) -> &'a str {
  return captures.get(index)
    .unwrap()
    .as_str()
}

// Note that index 0 containes the whole capture (string matching)
pub fn extract_capture_string(captures: &Captures, index: usize) -> String {
  return extract_capture_str(captures, index).to_owned()
}

pub fn get_link_info(link_str: &str) -> Result<LinkInfo, String> {
  /* Matches subproject_name::lib_name
    Capture 1: subproject_name
    Capture 2: lib_name
  */
  let single_link_matcher = Regex::new(r"^([a-zA-z0-9_\-.]+)::([a-zA-z0-9_\-.]+)$").unwrap();

  /* 
    Matches subproject_name::{ lib_name }
    Matches subproject_name::{ lib_name, another_lib_name }

    Capture 1: subproject_name
    Capture 2: { lib_name, another_lib_name }
    
    The second capture matches the whole list including brackets.
  */
  let mutli_link_matcher = Regex::new(r"^([a-zA-z0-9_\-.]+)::(\{ ?(?:[a-zA-z0-9_\-.]+, ?)*[a-zA-z0-9_\-.]+ ?\})$").unwrap();


  if let Some(captures) = single_link_matcher.captures(link_str) {
    return Ok(LinkInfo {
      from_project_name: extract_capture_string(&captures, 1),
      library_names: vec![extract_capture_string(&captures, 2)]
    });
  }
  else if let Some(captures) = mutli_link_matcher.captures(link_str) {
    let mut lib_links_list: &str = extract_capture_str(&captures, 2);

    {
      let open_bracket_index: usize = lib_links_list.find('{').unwrap();
      let close_bracket_index: usize = lib_links_list.rfind('}').unwrap();

      lib_links_list = (&lib_links_list[open_bracket_index + 1 .. close_bracket_index]).trim();
    }
    
    return Ok(LinkInfo {
      from_project_name: extract_capture_string(&captures, 1),
      library_names: lib_links_list.split(',')
        .map(|lib_name| lib_name.trim().to_owned())
        .collect()
    });
  }

  return Err(format!("Link specifier \"{}\" is in an invalid format", link_str));
}

pub fn validate_output_config(project_data: &RawProject) -> Result<(), String> {
  let mut makes_executable: bool = false;
  let mut makes_library: bool = false;

  for (output_name, output_data) in project_data.get_output() {
    if output_data.is_library_type() {
      if makes_library {
        return Err(format!("Project \"{}\" contains more than one library output, but should only contain one.", project_data.get_name()));
      }

      makes_library = true;

      if makes_executable {
        break;
      }
    }
    else if output_data.is_executable_type() {
      makes_executable = true;

      if makes_library {
        break
      }
    }
  }

  return if makes_executable && makes_library {
    Err(format!("Project \"{}\" should not create both library and executable outputs.", project_data.get_name()))
  }
  else {
    Ok(())
  }
}

pub fn validate_raw_project(project_data: &RawProject) -> Result<(), String> {
  if let Err(message) = validate_output_config(project_data) {
    return Err(message)
  }

  Ok(())
}
