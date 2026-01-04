//! POSIX path model implementation

use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use crate::PathModel;

pub enum StdFs {}



impl PathModel for StdPathModel {
  type Buf = PathBuf;
  type View<'path> = &'path Path;
  type Root = PathBuf;
  type RootView<'path> = &'path Path;
  type Link = PathBuf;
  type LinkView<'path> = &'path Path;
}
