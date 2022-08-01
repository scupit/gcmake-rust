use std::{path::{PathBuf, Path}, fs, io};

use super::{raw_data_in::{RawProject, RawSubproject, OutputItemType, RawTestProject}, path_manipulation::cleaned_pathbuf, final_project_data::ProjectLoadFailureReason};

#[derive(PartialEq, Eq)]
pub enum RetrievedCodeFileType {
  Source,
  Header,
  TemplateImpl,
  // Module (when implemented in compilers and build systems)
  Unknown
}

pub fn retrieve_file_type(any_path_type: impl AsRef<Path>) -> RetrievedCodeFileType {
  let the_path: &Path = any_path_type.as_ref();

  return match the_path.extension() {
    Some(extension) => match extension.to_str().unwrap() {
      "c" | "cpp" | "cxx" => RetrievedCodeFileType::Source,
      "h" | "hpp" | "hxx" => RetrievedCodeFileType::Header,
      "tpp" | "txx"       => RetrievedCodeFileType::TemplateImpl,
      _                   => RetrievedCodeFileType::Unknown
    },
    None => RetrievedCodeFileType::Unknown
  }
}

fn file_variants(
  project_root: &str,
  file_name_no_extensions: &str,
  possible_extensions: Vec<&str>
) -> Vec<PathBuf> {
  let base_file_path: PathBuf = Path::new(project_root).join(file_name_no_extensions);

  return possible_extensions
    .iter()
    .map(|extension| base_file_path.with_extension(extension))
    .collect();
}

pub fn yaml_names_from_dir(project_root: &str) -> Vec<PathBuf> {
  return file_variants(project_root, "cmake_data", vec!["yaml"]);
}

pub enum PrebuildScriptFile {
  Exe(PathBuf),
  Python(PathBuf)
}

pub fn find_prebuild_script(project_root: &str) -> Option<PrebuildScriptFile> {
  let pre_build_file_base_name: &str = "pre-build";

  for possible_exe_file in file_variants(project_root, pre_build_file_base_name, vec!["c", "cxx", "cpp"]) {
    if Path::exists(possible_exe_file.as_path()) {
      return Some(PrebuildScriptFile::Exe(cleaned_pathbuf(possible_exe_file)));
    }
  }

  for possible_python_file in file_variants(project_root, pre_build_file_base_name, vec!["py"]) {
    if Path::exists(possible_python_file.as_path()) {
      return Some(PrebuildScriptFile::Python(cleaned_pathbuf(possible_python_file)));
    }
  }

  return None
}

type YamlParseResult<T> = Result<T, ProjectLoadFailureReason>;

fn yaml_parse_helper<T: serde::de::DeserializeOwned>(project_root: &str) -> YamlParseResult<T> {
  for possible_cmake_data_file in yaml_names_from_dir(project_root) {
    if let io::Result::Ok(cmake_data_yaml_string) = fs::read_to_string(possible_cmake_data_file) {

      return match serde_yaml::from_str::<T>(&cmake_data_yaml_string) {
        Ok(serialized_project) => Ok(serialized_project),
        Err(error) => Err(ProjectLoadFailureReason::Other(error.to_string()))
      }
    }
  }

  return Err(ProjectLoadFailureReason::MissingYaml(format!(
    "Unable to find a cmake_data.yaml or cmake_data.yml file in {}",
    project_root
  )));
}

pub fn parse_root_project_data(project_root: &str) -> YamlParseResult<RawProject> {
  yaml_parse_helper(project_root)
}

pub fn parse_subproject_data(project_root: &str) -> YamlParseResult<RawSubproject> {
  yaml_parse_helper(project_root)
}

pub fn parse_test_project_data(project_root: &str) -> YamlParseResult<RawTestProject> {
  yaml_parse_helper(project_root)
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

pub enum ProjectOutputType {
  ExeProject,
  CompiledLibProject,
  HeaderOnlyLibProject
}

pub fn validate_raw_project_outputs(raw_project: &RawProject) -> Result<ProjectOutputType, String> {
  let mut num_exes: i32 = 0;
  let mut num_compiled_libs: i32 = 0;
  let mut num_header_only_libs: i32 = 0;

  for (_, raw_output_data) in raw_project.get_output() {
    match *raw_output_data.get_output_type() {
      OutputItemType::Executable => num_exes += 1,
      OutputItemType::HeaderOnlyLib => num_header_only_libs += 1,
      OutputItemType::CompiledLib
      | OutputItemType::StaticLib
      | OutputItemType::SharedLib => num_compiled_libs += 1
    }
  }

  let total_num_libs: i32 = num_compiled_libs + num_header_only_libs;

  if num_exes > 0 && total_num_libs > 0 {
    return Err(format!(
      "Project \"{}\" should not create both library and executable outputs.",
      raw_project.get_name()
    ));
  }
  else if total_num_libs > 1 {
    return Err(format!(
      "Project \"{}\" contains {} library outputs, but should only contain one.",
      total_num_libs,
      raw_project.get_name()
    ));
  }
  else if total_num_libs + num_exes == 0 {
    return Err(format!(
      "Project \"{}\" does not contain any output items. Each project is required to define at least one output.",
      raw_project.get_name()
    ));
  }

  return if num_compiled_libs == 1 {
    Ok(ProjectOutputType::CompiledLibProject)
  }
  else if num_header_only_libs == 1 {
    Ok(ProjectOutputType::HeaderOnlyLibProject)
  }
  else {
    // No libraries are created, and 1 or more executables are made
    Ok(ProjectOutputType::ExeProject)
  }
}
