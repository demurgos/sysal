//! POSIX path model implementation

use std::ffi::{CStr, CString};
use std::path::PathBuf;
use crate::PathModel;

pub struct Utf8PathBuf {
  inner: String,
}

pub struct Utf8PathView {
  inner: str
}

pub enum Utf8PathModel {}

impl PathModel for Utf8PathModel {
  type Buf = Utf8PathBuf;
  type View<'path> = &'path Utf8PathView;
  type Root = ();
  type RootView<'path> = ();
  type Link = String;
  type LinkView<'path> = &'path str;
}
