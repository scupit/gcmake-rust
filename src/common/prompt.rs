use std::{io::{self, stdin, Write}, convert::TryInto};

pub fn prompt_until_custom<F, T>(prompt: &str, transforming_predicate: F) -> io::Result<T>
  where F: Fn(&str) -> Option<T>
{
  let mut buffer = String::new();

  loop {
    buffer.clear();

    print!("{}: ", prompt);
    io::stdout().flush()?;
    stdin().read_line(&mut buffer)?;

    if let Some(success_value) = (transforming_predicate)(buffer.trim()) {
      return Ok(success_value);
    }
  }
}

pub fn prompt_until_custom_or_default<F, T: Clone>(
  prompt: &str,
  transforming_predicate: F,
  default_value: T,
  default_value_string: impl AsRef<str>
) -> io::Result<T>
  where F: Fn(&str) -> Option<T>
{
  prompt_until_custom(
    &format!("{} [{}]", prompt, default_value_string.as_ref()),
    |value| {
      if value.is_empty() {
        Some(default_value.clone())
      }
      else {
        transforming_predicate(value)
      }
    }
  )
}

pub fn prompt_once(prompt: &str) -> io::Result<String> {
  prompt_until_satisfies(prompt, |_| true)
}

pub fn prompt_until_not_empty(prompt: &str) -> io::Result<String> {
  prompt_until_satisfies(prompt, |value| !value.is_empty())
}

pub fn prompt_until_satisfies<F>(prompt: &str, predicate: F) -> io::Result<String>
  where F: Fn(&str) -> bool
{
  prompt_until_custom(
    prompt,
    |value| {
      if predicate(value) {
        Some(value.to_string())
      }
      else {
        None
      }
    }
  )
}

pub fn prompt_until_satisfies_or_default<F>(
  prompt: &str,
  predicate: F,
  default_value: impl AsRef<str>
) -> io::Result<String>
  where F: Fn(&str) -> bool
{
  prompt_until_custom_or_default(
    prompt,
    |value| {
      if predicate(value) {
        Some(value.to_string())
      }
      else {
        None
      }
    },
    default_value.as_ref().to_string(),
    default_value.as_ref()
  )
}

fn resolve_boolean_from_str(value_str: &str) -> Option<bool> {
  return match value_str {
    "y" => Some(true),
    "n" => Some(false),
    _ => None
  }
}

pub fn prompt_until_boolean_or_default(prompt: &str, default_value: bool) -> io::Result<bool> {
  let value_str: &str = if default_value
    { "y" }
    else { "n" };

  prompt_until_custom_or_default(
    prompt,
    resolve_boolean_from_str,
    default_value,
    value_str
  )
}

pub fn prompt_until_boolean(prompt: &str) -> io::Result<bool> {
  prompt_until_custom(
    &format!("{} (y or n)", prompt),
    resolve_boolean_from_str
  )
}

pub trait ChoiceResolver<T> {
  fn resolve_choice(&self) -> io::Result<T>;
}

impl<T, F> ChoiceResolver<T> for F
  where F: Fn() -> io::Result<T>
{
  fn resolve_choice(&self) -> io::Result<T> {
    self()
  }
}

pub struct ChoiceValue<T: Clone>(pub T);

impl<T: Clone> ChoiceResolver<T> for ChoiceValue<T> {
  fn resolve_choice(&self) -> io::Result<T> {
    Ok(self.0.clone())
  }
}

type PromptChoice<'a, T> = (&'a str, &'a dyn ChoiceResolver<T>);

pub fn prompt_with_choices<T>(
  prompt_title: &str,
  choices: &[PromptChoice<T>]
) -> io::Result<T> {
  let choice_list_string: String = choices
    .iter()
    .enumerate()
    .map(|(index, (choice_name, _))|
      format!("{}: {}\n", index + 1, choice_name)
    )
    .collect();
    
  return prompt_until_custom(
    &format!("{}{}", choice_list_string, prompt_title),
    |value| match value.parse::<usize>() {
      Ok(index) if index > 0 && index <= choices.len() => {
        let (_, value_resolver) = choices[index-1];
        Some(value_resolver.resolve_choice())
      },
      _ => None
    }
  )?;
}
