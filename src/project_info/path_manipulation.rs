use std::{path::{PathBuf, Path, Component}, io, env};

// This is a slightly modified version of the clean(...) function from path_clean:
// https://github.com/danreeves/path-clean/blob/master/src/lib.rs
fn internal_cleaned_pathbuf<P : AsRef<Path>>(unclean_path: P) -> PathBuf {
  let mut output: Vec<Component> = Vec::new();

  // https://doc.rust-lang.org/stable/std/path/enum.Component.html
  for comp in unclean_path.as_ref().components() {
    match comp {
      // Any "." can be ignored safely because root directories, drive prefixes,
      // and other important leading paths are already categorized and prepended
      // to output if encountered.
      Component::CurDir => (),
      Component::ParentDir => match output.last() {
        // Don't remove the root directory.
        Some(Component::RootDir) => (),
        Some(Component::Normal(_)) => { output.pop(); },
        _ => { output.push(comp); }
      },
      _ => output.push(comp)
    }
  }

  if output.is_empty() {
    // 'output' will only ever be empty if no drive prefix or root directory
    // were ever encountered, and the rest of the path was only made up of "."s.
    return PathBuf::from(".");
  }
  return output.iter().collect();
}

pub fn cleaned_pathbuf<P : AsRef<Path>>(unclean_path: P) -> PathBuf {
  internal_cleaned_pathbuf(unclean_path)
}

pub fn cleaned_path_str<P : AsRef<Path>>(file_path: P) -> String {
  return cleaned_pathbuf(file_path).to_str().unwrap().to_string();
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

pub fn relative_to_project_root(project_root: &Path, file_path: impl AsRef<Path>) -> PathBuf {
  return cleaned_pathbuf(file_path.as_ref().strip_prefix(project_root).unwrap());
}

pub fn unix_style(path: impl AsRef<Path>) -> String {
  return path.as_ref().to_str().unwrap().replace("\\", "/");
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

// This is a slightly modified version of the test suite from path_clean:
// https://github.com/danreeves/path-clean/blob/master/src/lib.rs
// Mainly, I've added some tests and made the Windows-style tests run on all platforms.
#[cfg(test)]
mod tests {
  use super::cleaned_pathbuf;
  use std::path::PathBuf;

  fn check_transformation_results(test_cases: &Vec<(&str, &str)>) {
    for (input, expected_output) in test_cases {
      assert_eq!(cleaned_pathbuf(input), PathBuf::from(expected_output));
    }
  }

  #[test]
  fn test_empty_path_is_current_dir() {
    assert_eq!(cleaned_pathbuf(""), PathBuf::from("."));
  }

  #[test]
  fn test_clean_paths_dont_change() {
    let test_cases = vec![
      (".", "."),
      ("..", ".."),
      ("/", "/"),
      ("\\", "\\"),
      ("\\\\", "\\\\")
    ];

    check_transformation_results(&test_cases);
  }

  #[test]
  fn test_replace_multiple_slashes() {
    let test_cases = vec![
      ("/", "/"),
      ("//", "/"),
      ("///", "/"),
      (".//", "."),
      ("//..", "/"),
      ("//..//", "/"),
      ("..//", ".."),
      ("..//./", ".."),
      ("..//./..///", "../.."),
      ("/..//", "/"),
      ("/.//./", "/"),
      ("././/./", "."),
      ("path//to///thing", "path/to/thing"),
      ("/path//to///thing", "/path/to/thing"),
    ];
  
    check_transformation_results(&test_cases);
  }

  #[test]
  fn test_eliminate_current_dir() {
    let test_cases = vec![
      ("./", "."),
      ("/./", "/"),
      ("./test", "test"),
      ("./test/./path", "test/path"),
      ("/test/./path/", "/test/path"),
      ("test/path/.", "test/path"),
    ];

    check_transformation_results(&test_cases);
  }

  #[test]
  fn test_eliminate_parent_dir() {
    let test_cases = vec![
      ("/..", "/"),
      ("/../test", "/test"),
      ("test/..", "."),
      ("test/path/..", "test"),
      ("test/../path", "path"),
      ("/test/../path", "/path"),
      ("test/path/../../", "."),
      ("test/path/../../..", ".."),
      ("/test/path/../../..", "/"),
      ("/test/path/../../../..", "/"),
      ("test/path/../../../..", "../.."),
      ("test/path/../../another/path", "another/path"),
      ("test/path/../../another/path/..", "another"),
      ("../test", "../test"),
      ("../test/", "../test"),
      ("../test/path", "../test/path"),
      ("../test/..", ".."),
    ];

    check_transformation_results(&test_cases);
  }

  #[test]
  #[cfg(target_os = "windows")]
  fn test_windows_paths() {
    let test_cases = vec![
      ("C:\\test\\path/.", "C:/test/path"),
      (".\\", "."),
      ("\\..", "\\"),
      ("\\..\\test", "\\test"),
      ("\\\\remote\\path\\.", "\\\\remote\\path"),
      ("\\\\..", "\\\\"),
      ("\\\\remote\\..\\..", "\\\\"),
      ("test\\..", "."),
      ("test\\path\\..\\..\\..", ".."),
      ("test\\path/..\\../another\\path", "another\\path"),
      ("test\\path\\my/path", "test\\path\\my\\path"),
      ("/dir\\../otherDir/test.json", "/otherDir/test.json"),
      ("c:\\test\\..", "c:\\"),
      ("c:/test/..", "c:/")
    ];

    check_transformation_results(&test_cases);
  }
}
