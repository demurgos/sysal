use std::ffi::CString;
use crate::mode::Mode;

/// Windows-32 semantics
///
/// - case-insensitive, using ASCII equivalence
/// - first write wins on duplicate
pub enum W32 {}

impl Mode for W32 {
  type Variable = CString;
}