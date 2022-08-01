use super::{RawMutualExclusionSet, RawPredefinedTargetMapIn};


pub trait RawPredepCommon {
  fn maybe_mutual_exclusion_groups(&self) -> &Option<RawMutualExclusionSet>;
  fn raw_target_map_in(&self) -> &RawPredefinedTargetMapIn;
}