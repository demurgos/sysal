#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct NodeId(usize);

impl NodeId {
  pub const ZERO: Self = Self(0);

  pub const fn successor(self) -> Option<Self> {
    match self.0.checked_add(1) {
      None => None,
      Some(id) => Some(Self(id)),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NodeIdGenerator {
  next: Option<NodeId>,
}

impl NodeIdGenerator {
  pub const fn new() -> Self {
    Self { next: Some(NodeId::ZERO) }
  }
}

impl Iterator for NodeIdGenerator {
  type Item = NodeId;

  fn next(&mut self) -> Option<Self::Item> {
    let result = self.next;
    self.next = match result {
      Some(r) => r.successor(),
      None => None,
    };
    result
  }
}

/// In-memory file system backed by `BTree`
pub struct VecFs {

}

struct VecDir {

}


