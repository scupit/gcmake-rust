pub mod prompt;

pub fn make_c_identifier(item: impl AsRef<str>) -> String {
  return item.as_ref()
    .replace(" ", "_")
    .replace("-", "_");
}

pub fn basic_configure_replace<'a>(
  the_str: impl AsRef<str>,
  replacements: impl IntoIterator<Item=(&'a str, String)>
) -> String {
  let mut final_string: String = the_str.as_ref().to_string();

  for (to_replace, replacement_text) in replacements {
    final_string = final_string.replace(
      &format!("@{}@", to_replace),
      &replacement_text
    )
  }

  return final_string;
}
