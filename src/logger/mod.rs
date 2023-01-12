use std::process::exit;
use colored::Colorize;

pub fn warn(message: impl AsRef<str>) {
  println!(
    "{}: {}",
    "Warning".yellow(),
    message.as_ref()
  );
}

pub fn exit_error_log(error_message: impl AsRef<str>) -> ! {
  eprintln!("{}", error_message.as_ref());
  exit(0);
}