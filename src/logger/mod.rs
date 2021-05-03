use std::process::exit;


pub fn exit_error_log(error_message: &str) {
  eprintln!("{}", error_message);
  exit(0);
}