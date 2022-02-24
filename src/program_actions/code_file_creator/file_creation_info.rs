use crate::{project_info::path_manipulation::{relative_to_project_root, cleaned_path_str}, cli_config::FileCreationLang};

pub struct FileTypeGeneratingInfo {
  pub generating_header: bool,
  pub generating_source: bool,
  pub generating_template_impl: bool
}

impl FileTypeGeneratingInfo {
  pub fn new(specifier: &str) -> Result<Self, String> {
    let mut info = Self {
      generating_header: false,
      generating_source: false,
      generating_template_impl: false
    };

    for char in specifier.chars() {
      match char {
        'h' => info.generating_header = true,
        's' => info.generating_source = true,
        't' => info.generating_template_impl = true,
        invalid_char => return Err(format!(
          "Invalid character '{}' in file type specifier. Only 'h', 's', and 't' are allowed.",
          invalid_char
        ))
      }
    }

    return Ok(info);
  }

  pub fn only_generating_template_impl(&self) -> bool {
    self.generating_template_impl && !(
      self.generating_header || self.generating_source
    )
  }

  pub fn only_generating_source(&self) -> bool {
    self.generating_source && !(
      self.generating_template_impl || self.generating_header
    )
  }
}

pub enum FileGuardStyle {
  IncludeGuard(String),
  PragmaOnce
}

pub struct SharedFileInfo {
  pub shared_name: String,
  pub leading_dir_path: String,
  pub cleaned_given_path: String
}

impl SharedFileInfo {
  pub fn new(
    file_class_name: &str,
    project_root: &str
  ) -> Self {
    let cleaned_given_path: String = relative_to_project_root(
      project_root,
      cleaned_path_str(file_class_name).into()
    );

    return if let Some(last_slash_index) = cleaned_given_path.rfind('/') {
      Self {
        shared_name: String::from(&cleaned_given_path[last_slash_index + 1..]),
        leading_dir_path: String::from(&cleaned_given_path[0..last_slash_index]),
        cleaned_given_path
      }
    }
    else {
      Self {
        shared_name: cleaned_given_path.clone(),
        cleaned_given_path,
        leading_dir_path: String::from("."),
      }
    }
  }
}

pub fn validate_which_generating(
  lang: &FileCreationLang,
  which_generating: &FileTypeGeneratingInfo
) -> Result<(), String> {
  if which_generating.only_generating_source() {
    return Err(String::from("Error: For now, generating only the source file is not supported."));
  }
  else if which_generating.only_generating_template_impl() {
    return Err(String::from("Error: For now, generating only the template-implementation file is not supported."));
  }

  match *lang {
    FileCreationLang::C => {
      if which_generating.generating_template_impl {
        return Err(String::from("Error: Cannot generate a template implementation file (.tpp) for the C language. Please remove 't' from the file types specifier."));
      }
    }
    FileCreationLang::Cpp => { }
  }

  Ok(())
}

pub fn validate_shared_file_info(shared_info: &SharedFileInfo) -> Result<(), String> {
  if shared_info.shared_name.contains('.') {
    return Err(format!(
      "Given file name '{}' should not have an extension, but does. Please remove the extension from {}",
      shared_info.shared_name,
      shared_info.cleaned_given_path
    ));
  }

  Ok(())
}
