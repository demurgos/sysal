//! File system abstraction layer

pub trait FsRead {
  fn get(&self, path: &str) -> &[u8];
}
