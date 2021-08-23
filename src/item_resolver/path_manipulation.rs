use std::path::{Path, PathBuf};

pub enum CodeFileType {
  Header,
  Source,
  TemplateImpl
}

pub fn cleaned_path_str(file_path: &str) -> String {
  return path_clean::clean(&file_path.replace("\\", "/"));
}

pub fn cleaned_pathbuf(file_path: PathBuf) -> PathBuf {
  let replaced_path: String = cleaned_path_str(file_path.to_str().unwrap());
  return PathBuf::from(replaced_path);
}

pub fn cleaned_path(file_path: &Path) -> PathBuf {
  let replaced_path: String = cleaned_path_str(file_path.to_str().unwrap());
  return PathBuf::from(replaced_path);
}

pub fn determine_file_type(file_name: &str) -> Result<CodeFileType, String> {
  return if let Some(extension) = Path::new(file_name).extension() {
    match extension.to_str().unwrap() {
      "c" | "cpp" | "cxx" | "c++" => Ok(CodeFileType::Source),
      "h" | "hpp" | "hxx" | "h++" => Ok(CodeFileType::Header),
      "t" | "tpp" | "txx" | "t++" => Ok(CodeFileType::TemplateImpl),
      ext => Err(format!("Invalid extension '{}' for file '{}'", ext, file_name))
    }
  }
  else {
    Err(format!("File '{}' is missing a mandatory extension.", file_name))
  }
}
