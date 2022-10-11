use std::{path::{PathBuf, Path}, fs::{self, DirEntry}, io, env};

pub fn cleaned_path_str(file_path: &str) -> String {
  return path_clean::clean(&file_path.replace("\\", "/"));
}

pub fn cleaned_pathbuf(file_path: PathBuf) -> PathBuf {
  let replaced_path: String = cleaned_path_str(file_path.to_str().unwrap());
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

fn absolute_path_internal<T>(a_path: &T) -> io::Result<PathBuf>
  where T: AsRef<Path> + ToString
{
  let abs_path: PathBuf = cleaned_pathbuf(env::current_dir()?.join(a_path));
  return Ok(abs_path);
}

pub fn absolute_path<T>(a_path: T) -> Result<PathBuf, String>
  where T: AsRef<Path> + ToString
{
  match absolute_path_internal(&a_path) {
    Ok(abs_pathbuf) => Ok(abs_pathbuf),
    Err(err) => Err(format!(
      "Failed to resolve absolute path from '{}'. More details: {}",
      a_path.to_string(),
      err.to_string())
    )
  }
}

pub fn find_first_dir_named(
  root: &Path,
  looking_for: &str
) -> io::Result<Option<PathBuf>> {
  for entry in fs::read_dir(root)? {
    let entry: DirEntry = entry?;
    let entry_path: PathBuf = entry.path();

    if entry_path.is_dir() {
      if entry.file_name() == looking_for {
        return Ok(Some(entry_path))
      }

      if let Some(matching_path) = find_first_dir_named(entry_path.as_path(), looking_for)? {
        return Ok(Some(matching_path))
      }
    }
  }

  Ok(None)
}
