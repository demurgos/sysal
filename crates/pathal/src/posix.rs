//! POSIX path model implementation

use std::ffi::{CStr, CString};
use std::path::PathBuf;

pub struct PosixPathBuf {
  inner: CString,
}

pub struct PosixPath {
  inner: CStr
}

pub trait PathModel {
  type Buf;
  type Root;
  type Link;
}

pub struct AbsolutePath(pub PathBuf);

pub enum AnyCommand<TyModel: PathModel> {
  Current,
  Parent,
  Root(TyModel::Root),
  Link(TyModel::Link),
}

pub struct AnyPath<TyModel: PathModel> {
  commands: Vec<AnyCommand<TyModel>>
}
