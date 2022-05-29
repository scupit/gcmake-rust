use std::process::exit;


pub fn exit_error_log(error_message: impl AsRef<str>) -> ! {
  eprintln!("{}", error_message.as_ref());
  exit(0);
}