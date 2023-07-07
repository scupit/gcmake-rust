use std::{path::{PathBuf, Path}, fs::{self}, io::{self}, collections::BTreeSet};

use colored::Colorize;
use regex::Regex;

use super::{raw_data_in::{RawProject, RawSubproject, OutputItemType, RawTestProject}, path_manipulation::{cleaned_pathbuf, relative_to_project_root}, final_project_data::{ProjectLoadFailureReason, CppFileGrammar}, CodeFileInfo};

#[derive(Clone, Copy)]
pub enum CodeFileLang {
  C,
  Cpp {
    used_grammar: CppFileGrammar
  },
  Cuda
}

#[derive(Clone)]
pub enum RetrievedCodeFileType {
  Source(CodeFileLang),
  Header(CodeFileLang),
  TemplateImpl,
  // Module (when implemented in compilers and build systems)
  Unknown
}

impl RetrievedCodeFileType {
  pub fn lang(&self) -> Option<CodeFileLang> {
    return match self {
      RetrievedCodeFileType::Header(lang) => Some(lang.clone()),
      RetrievedCodeFileType::Source(lang) => Some(lang.clone()),
      RetrievedCodeFileType::TemplateImpl => Some(CodeFileLang::Cpp { used_grammar: CppFileGrammar::Cpp1 }),
      RetrievedCodeFileType::Unknown => None
    }
  }

  pub fn is_source(&self) -> bool {
    match self {
      Self::Source { .. } => true,
      _ => false
    }
  }

  pub fn is_normal_header(&self) -> bool {
    match self {
      Self::Header(_) => true,
      _ => false
    }
  }

  pub fn is_same_general_type_as(&self, other: &RetrievedCodeFileType) -> bool {
    match (self, other) {
      (Self::Source { .. }, Self::Source { .. }) => true,
      (Self::Header(_), Self::Header(_)) => true,
      (Self::TemplateImpl, Self::TemplateImpl) => true,
      _ => false
    }
  }
}

pub fn code_file_type(any_path_type: impl AsRef<Path>) -> RetrievedCodeFileType {
  let the_path: &Path = any_path_type.as_ref();

  return match the_path.extension() {
    Some(extension) => match extension.to_str().unwrap() {
      "cpp2"                          => RetrievedCodeFileType::Source(CodeFileLang::Cpp { used_grammar: CppFileGrammar::Cpp2 }),
      "c"                             => RetrievedCodeFileType::Source(CodeFileLang::C),
      "cc"| "cpp" | "cxx"             => RetrievedCodeFileType::Source(CodeFileLang::Cpp { used_grammar: CppFileGrammar::Cpp1 }),
      "cu"                            => RetrievedCodeFileType::Source(CodeFileLang::Cuda),
      // NOTE: I'm treating ".h" headers as C only. Any C++ projects which use ".h" for
      // C++ headers will also contain C++ source files (therefore requiring a C++ language configuration
      // for the project), so this shouldn't cause any errors.
      "h"                             => RetrievedCodeFileType::Header(CodeFileLang::C),
      "hh" | "hpp" | "hxx"            => RetrievedCodeFileType::Header(CodeFileLang::Cpp { used_grammar: CppFileGrammar::Cpp1 }),
      "cuh"                           => RetrievedCodeFileType::Header(CodeFileLang::Cuda),
      "tpp" | "tcc" | "txx" | "inl"   => RetrievedCodeFileType::TemplateImpl,
      _                               => RetrievedCodeFileType::Unknown
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

fn first_existing_variant(
  project_root: &str,
  file_name_no_extensions: &str,
  possible_extensions: Vec<&str>
) -> Option<PathBuf> {
  return file_variants(project_root, file_name_no_extensions, possible_extensions)
    .into_iter()
    .find(|possible_file| possible_file.exists());
}

pub fn yaml_names_from_dir(project_root: &str) -> Vec<PathBuf> {
  return file_variants(project_root, "cmake_data", vec!["yaml"]);
}

pub enum PrebuildScriptFile {
  Exe(PathBuf),
  Python(PathBuf)
}

pub fn find_doxyfile_in(project_docs_dir: &str) -> Option<PathBuf> {
  return first_existing_variant(project_docs_dir, "Doxyfile", vec!["in"])
}

pub struct SphinxConfigFiles {
  pub index_rst: Option<PathBuf>,
  pub conf_py_in: Option<PathBuf>
}

pub fn find_sphinx_files(project_docs_dir: &str) -> SphinxConfigFiles {
  return SphinxConfigFiles {
    index_rst: first_existing_variant(project_docs_dir, "index", vec!["rst"]),
    conf_py_in: first_existing_variant(project_docs_dir, "conf", vec!["py.in"]),
  };
}

type AssignmentCheckGroup<'a> = (&'a str, &'a str);

fn validate_assignments_in_contents(
  file_path: &Path,
  file_contents: &str,
  at_replacements: Vec<AssignmentCheckGroup>
) -> Result<(), String> {
  // TODO: Make this "search" more efficient.
  for (field_name, required_text) in at_replacements {
    let finder_regex_string: String = format!(r"(?m)^\s*{}\s*=\s*{}\s*", field_name, required_text);
    let finder_regex: Regex = Regex::new(&finder_regex_string).unwrap();

    if !finder_regex.is_match(&file_contents) {
      return Err(format!(
        "{} is missing the line `{}`, which is required for it to work properly with CMake.",
        file_path.to_str().unwrap().yellow(),
        format!("{} = {}", field_name, required_text).bright_green()
      ));
    }
  }

  return Ok(())
}

pub fn validate_conf_py_in(conf_py_in_path: &Path) -> Result<(), String> {
  let conf_py_in_contents: String = fs::read_to_string(conf_py_in_path)
    .map_err(|err| format!(
      "Error reading {}: {}",
      conf_py_in_path.to_str().unwrap(),
      err.to_string()
    ))?;

  let assignment_pairs: Vec<AssignmentCheckGroup> = vec![
    ("project", "\"@PROJECT_NAME@\""),
    ("author", "\"@PROJECT_VENDOR@\""),
    ("release", "\"@PROJECT_VERSION@\""),
    ("breathe_default_project", "\"@PROJECT_NAME@\""),
  ];

  return validate_assignments_in_contents(
    conf_py_in_path,
    &conf_py_in_contents,
    assignment_pairs
  );
}

pub fn validate_doxyfile_in(doxyfile_in_path: &Path) -> Result<(), String> {
  let doxyfile_in_contents: String = fs::read_to_string(doxyfile_in_path)
    .map_err(|err| format!(
      "Error reading {}: {}",
      doxyfile_in_path.to_str().unwrap(),
      err.to_string()
    ))?;

  // Map Doxyfile fields to variables provided in the CMakeLists.txt.
  let at_replacements: Vec<AssignmentCheckGroup> = vec![
    ("PROJECT_NAME", "\"@PROJECT_NAME@\""),
    ("PROJECT_NUMBER", "\"@PROJECT_VERSION@\""),
    ("PROJECT_BRIEF", "\"@PROJECT_DESCRIPTION@\""),
    ("OUTPUT_DIRECTORY", "\"@DOXYGEN_OUTPUT_DIR@\""),
    ("INPUT", "@DOXYGEN_INPUTS@"),
    ("GENERATE_HTML", "YES"),
    ("HTML_OUTPUT", "html/"),
    ("HTML_FILE_EXTENSION", ".html"),
    ("GENERATE_XML", "YES"),
    ("XML_OUTPUT", "xml/"),
    ("ENABLE_PREPROCESSING", "YES"),
    ("MACRO_EXPANSION", "YES"),
    ("PREDEFINED", "@DOXYGEN_PREDEFINED_MACROS@"),
    ("STRIP_FROM_PATH", "\"..\""),
  ];

  return validate_assignments_in_contents(
    doxyfile_in_path,
    &doxyfile_in_contents,
    at_replacements
  );
}

pub fn find_prebuild_script(project_root: &str) -> Option<PrebuildScriptFile> {
  let pre_build_file_base_name: &str = "pre_build";

  for possible_exe_file in file_variants(project_root, pre_build_file_base_name, vec!["c", "cxx", "cpp", "cpp2", "cu"]) {
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

pub fn populate_existing_files<F>(
  root_dir: &Path,
  current_dir_checking: &Path,
  file_list: &mut BTreeSet<CodeFileInfo>,
  filter_func: &F
) -> io::Result<()>
  where F: Fn(&Path) -> bool
{
  if current_dir_checking.is_dir() {
    for dirent in fs::read_dir(current_dir_checking)? {
      let path: PathBuf = dirent?.path();

      if path.is_dir() {
        populate_existing_files(root_dir, &path, file_list, filter_func)?;
      }
      else if path.is_file() && filter_func(path.as_path()) {
        let file_info: CodeFileInfo = CodeFileInfo::from_path(
          cleaned_pathbuf(relative_to_project_root(root_dir.to_str().unwrap(), path.as_path())),
          false
        );

        // Rust sets don't overwrite existing values. Since generated files are always added to
        // file sets first, we don't ever overwrite the generated files.
        file_list.insert(file_info);
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
