//! POSIX path model implementation

use std::ffi::{CStr, CString};
use std::path::PathBuf;

pub struct PosixPathBuf {
  inner: CString,
}

pub struct PosixPath {
  inner: CStr
}

pub enum PosixPathModel {}
