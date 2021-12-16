use std::path::{PathBuf};

pub fn cleaned_path_str(file_path: &str) -> String {
  return path_clean::clean(&file_path.replace("\\", "/"));
}

pub fn cleaned_pathbuf(file_path: PathBuf) -> PathBuf {
  let replaced_path: String = cleaned_path_str(file_path.to_str().unwrap());
  return PathBuf::from(replaced_path);
}

pub fn relative_to_project_root(project_root: &str, file_path: PathBuf) -> String {
  let replacer: String = if project_root == "." {
    "./".to_owned()
  }
  else if project_root.ends_with("/") {
    project_root.to_owned()
  }
  else {
    let mut owned: String = project_root.to_owned();
    owned.push_str("/");
    owned
  };

  return file_path
    .to_string_lossy()
    .to_string()
    .replace(&replacer, "");
}
