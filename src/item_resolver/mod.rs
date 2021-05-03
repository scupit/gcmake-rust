mod dependency_handling;
use crate::data_types::raw_types::*;

use self::dependency_handling::DependencyGraph;

pub struct FinalProjectData {
  project_root: String,
  project: RawProject,
  dependency_graph: DependencyGraph
}

impl FinalProjectData {
  pub fn new(
    project_root: String,
    project: RawProject
  ) -> FinalProjectData {
    let ref src_dir = format!("{}/src/{}", project_root, project.get_name());
    let ref include_dir = format!("{}/include/{}", project_root, project.get_name());
    let ref template_impls_dir = format!("{}/template-impl/{}", project_root, project.get_name());

    let dependency_graph = DependencyGraph::new(
      &project_root,
      src_dir,
      include_dir,
      template_impls_dir,
      &project
    );

    return FinalProjectData {
      project_root,
      project,
      dependency_graph
    }
  }

  fn get_project_name(&self) -> &str {
    return self.project.get_name();
  }

  pub fn get_raw_project(&self) -> &RawProject {
    return &self.project;
  }
}