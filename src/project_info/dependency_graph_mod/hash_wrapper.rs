use std::{ops::Deref, cell::RefCell, hash::{Hash, Hasher}, rc::Rc};

pub struct RcRefcHashWrapper<T>(pub Rc<RefCell<T>>)
  where T: Hash + PartialEq + Eq;

impl<T> RcRefcHashWrapper<T>
  where T: Hash + PartialEq + Eq
{
  pub fn unwrap(self) -> Rc<RefCell<T>> {
    self.0
  }
}

impl<T> Deref for RcRefcHashWrapper<T>
  where T: Hash + PartialEq + Eq
{
  type Target = Rc<RefCell<T>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> Hash for RcRefcHashWrapper<T>
  where T: Hash + PartialEq + Eq
{
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.as_ref().borrow().hash(state)
  }
}

impl<T> Clone for RcRefcHashWrapper<T>
  where T: Hash + PartialEq + Eq
{
  fn clone(&self) -> Self {
    Self(Rc::clone(&self.0))
  }
}

impl<T> PartialEq for RcRefcHashWrapper<T>
  where T: Hash + PartialEq + Eq
{
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}


impl<T> Eq for RcRefcHashWrapper<T>
  where T: Hash + PartialEq + Eq
{ }

impl<T> Ord for RcRefcHashWrapper<T>
  where T: Hash + PartialEq + Eq + Ord
{
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.0.cmp(&other.0)
  }
}

impl<T> PartialOrd for RcRefcHashWrapper<T>
  where T: Hash + PartialEq + Eq + PartialOrd
{
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.0.partial_cmp(&other.0)
  }
}