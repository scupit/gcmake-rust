use super::{RawMutualExclusionSet, RawPredefinedTargetMapIn};

pub trait RawPredepCommon {
  fn can_trivially_cross_compile(&self) -> bool;
  fn maybe_mutual_exclusion_groups(&self) -> &Option<RawMutualExclusionSet>;
  fn raw_target_map_in(&self) -> &RawPredefinedTargetMapIn;
  fn repo_url(&self) -> Option<&str>;
  fn github_url(&self) -> Option<&str>;
}