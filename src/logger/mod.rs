use std::process::exit;
use colored::Colorize;

pub fn warn(message: impl AsRef<str>) {
  println!(
    "{}: {}",
    "Warning".yellow(),
    message.as_ref()
  );
}

pub fn block(closure: impl FnOnce()) {
  closure();
  println!("----------------------------------------");
}

pub fn exit_error_log(error_message: impl AsRef<str>) -> ! {
  block(|| {
    eprintln!("{}", error_message.as_ref());
  });
  exit(0);
}