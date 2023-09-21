use std::{io, collections::BTreeSet, rc::Rc, cell::RefCell, fs::File, path::{PathBuf, Path}};
use std::io::Write;

use crate::project_info::{dependency_graph_mod::dependency_graph::{DependencyGraphInfoWrapper, DependencyGraph}, final_dependencies::FinalDebianPackagesConfig};

const DEBIAN_DEP_INSTALL_SH_FILE_NAME: &'static str = "install-deb-development-packages.sh";

pub fn write_debian_dep_install_sh<'a>(
  dep_graph_wrapper: &'a DependencyGraphInfoWrapper<'a>
) -> io::Result<()> {
  let borrowed_project = dep_graph_wrapper.root_dep_graph.as_ref().borrow();

  if let Some(project_data) = borrowed_project.project_wrapper().maybe_normal_project() {
    let mut accumulated_deps: BTreeSet<String> = BTreeSet::new();

    recurse_for_dependencies(
      &dep_graph_wrapper.root_dep_graph,
      &mut accumulated_deps
    );

    let file_path: PathBuf = Path::new(project_data.get_project_root_relative_to_cwd()).join(DEBIAN_DEP_INSTALL_SH_FILE_NAME);
    let deb_dep_installer_file: File = File::create(file_path.as_path())?;

    if accumulated_deps.is_empty() {
      write!(&deb_dep_installer_file,
        "echo 'No system dependencies need to be installed for developing {}'",
        borrowed_project.project_identifier_name()
      )?;
    }
    else {
      write!(&deb_dep_installer_file,
        "apt install {}",
        accumulated_deps.into_iter().collect::<Vec<String>>().join(" ")
      )?;
    }
  }
  Ok(())
}

pub fn recurse_for_dependencies(
  root_graph: &Rc<RefCell<DependencyGraph>>,
  accumulated_deps: &mut BTreeSet<String>
) {
  let borrowed_graph = root_graph.as_ref().borrow();

  for (_, subproject_graph) in borrowed_graph.get_subprojects() {
    recurse_for_dependencies(subproject_graph, accumulated_deps);
  }

  for (_, gcmake_dep_graph) in borrowed_graph.get_gcmake_dependencies() {
    recurse_for_dependencies(gcmake_dep_graph, accumulated_deps);
  }

  for (_, predep_graph) in borrowed_graph.get_predefined_dependencies() {
    let debian_package_config: FinalDebianPackagesConfig = predep_graph.as_ref().borrow()
      .project_wrapper()
      .clone()
      .unwrap_predef_dep()
      .as_common()
      .debian_packages_config()
      .clone();

    for dev_package_name in debian_package_config.dev {
      accumulated_deps.insert(dev_package_name);
    }

    for runtime_package_name in debian_package_config.runtime {
      accumulated_deps.insert(runtime_package_name);
    }
  }
}