use std::{path::{PathBuf, Path}, io, env};

pub fn cleaned_path_str(file_path: &str) -> String {
  return path_clean::clean(&file_path.replace("\\", "/"));
}

pub fn cleaned_pathbuf(file_path: impl AsRef<Path>) -> PathBuf {
  let replaced_path: String = cleaned_path_str(file_path.as_ref().to_str().unwrap());
  return PathBuf::from(replaced_path);
}

pub fn without_leading_dot(some_path_or_extension: impl AsRef<str>) -> String {
  let the_str: &str = some_path_or_extension.as_ref();

  return if the_str.starts_with('.') {
    String::from(&the_str[1..])
  }
  else {
    String::from(the_str)
  }
}

pub fn relative_to_project_root(project_root: &str, file_path: impl AsRef<Path>) -> String {
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
    .as_ref()
    .to_string_lossy()
    .to_string()
    .replace(&replacer, "");
}

fn absolute_path_internal(a_path: &Path) -> io::Result<PathBuf>
{
  let abs_path: PathBuf = cleaned_pathbuf(env::current_dir()?.join(a_path));
  return Ok(abs_path);
}

pub fn absolute_path(a_path: impl AsRef<Path>) -> Result<PathBuf, String> {
  return absolute_path_internal(a_path.as_ref())
    .map_err(|err| format!(
      "Failed to resolve absolute path from '{}'. More details: {}",
      a_path.as_ref().to_str().unwrap(),
      err.to_string()
    ));
}
