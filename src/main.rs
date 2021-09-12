mod data_types;
mod item_resolver;
mod logger;
mod cmakelists_writer;

use item_resolver::FinalProjectData;
use logger::exit_error_log;

use crate::cmakelists_writer::write_cmakelists;

fn main() {
  let args: Vec<String> = std::env::args().collect();

  let project_root_dir = match args.len() {
    1 => ".",
    _ => &args[1]
  };


  match FinalProjectData::new(project_root_dir) {
    Ok(project_data) => {
      // println!("Project in YAML: {:?}", project_data.get_raw_project());

      // println!("Project root: {:?}", project_data.get_project_root());
      // println!("Include Dir: {:?}", project_data.get_include_dir());

      match write_cmakelists(&project_data) {
        Ok(_)=> println!("CMakeLists all written successfully!"),
        Err(err) => println!("{:?}", err)
      }

      // println!("src dir: {:?}", project_data.get_src_dir());
      // println!("src list: {:?}", project_data.src_files);

      // println!("header dir: {:?}", project_data.get_include_dir());
      // println!("header list: {:?}", project_data.include_files);

      // println!("template-impl dir: {:?}", project_data.get_template_impl_dir());
      // println!("template-impl list: {:?}", project_data.template_impl_files);
    },
    Err(message) => exit_error_log(&message)
  }
}
