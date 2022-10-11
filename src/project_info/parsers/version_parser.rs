pub type ThreePartVersionTuple = (u32, u32, u32);

pub struct ThreePartVersion {
  pub major: u32,
  pub minor: u32,
  pub patch: u32,
}

impl ThreePartVersion {
  pub fn to_string(&self) -> String {
    let Self { major, minor, patch } = self;

    format!("{}.{}.{}", major, minor, patch)
  }

  pub fn as_tuple(&self) -> ThreePartVersionTuple {
    return (self.major, self.minor, self.patch);
  }

  /*
    Allowed input formats:
      - v0.0.1
      - 0.0.1
  */
  pub fn from_str(full_version_string: &str) -> Option<Self> {
    let usable_version_string = if full_version_string.starts_with('v')
      { &full_version_string[1..] }
      else { full_version_string };

    let mut version_nums: Vec<Result<u32, _>> = usable_version_string
      .split('.')
      .map(|section| section.parse::<u32>())
      .collect();

    if version_nums.len() != 3 {
      return None;
    }

    for maybe_num in &version_nums {
      if maybe_num.is_err() {
        return None;
      }
    }

    return Some(Self {
      major: version_nums.remove(0).unwrap(),
      minor: version_nums.remove(0).unwrap(),
      patch: version_nums.remove(0).unwrap(),
    });
  }
}

pub fn parse_version(version_str: &str) -> Option<ThreePartVersion> {
  return ThreePartVersion::from_str(version_str);
}
