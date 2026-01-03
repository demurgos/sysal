use std::ffi::{CStr, CString};
use crate::mode::Mode;

/// GNU lib C semantics
///
/// - case-sensitive
/// - first write wins on duplicate
///
/// An environment implementation based on the semantics used by the GNU C
/// library, also known as _glibc_.
///
/// - [_glibc_ website](https://www.gnu.org/software/libc/)
/// - [_glibc_ repository](https://sourceware.org/git/glibc.git)
///
/// This implementation is based on the following files:
/// - [`posix/environ.c`](https://sourceware.org/git/?p=glibc.git;a=blob;f=posix/environ.c;h=924effe3cd65dc6903aa031b83876ebbdc88257d;hb=HEAD)
/// - [`stdlib/getenv.c`](https://sourceware.org/git/?p=glibc.git;a=blob;f=stdlib/getenv.c;h=1a7b0bfc063e5fa63cd8d4bf4b661c09c621a0fb;hb=HEAD)
/// - [`stdlib/setenv.c`](https://sourceware.org/git/?p=glibc.git;a=blob;f=stdlib/setenv.c;h=2b8b6cafa003236db5d4b44c1188e6992ed0c4ed;hb=HEAD)
/// - [`stdlib/setenv.h`](https://sourceware.org/git/?p=glibc.git;a=blob;f=stdlib/setenv.h;h=07ac97b9061fdc57a394bb0f2bc93de541057b23;hb=HEAD)
pub enum Glibc {}

impl Mode for Glibc {
  type Variable = CString;
}

pub enum InvalidKeyError {
  Empty,
  EqualSign(usize),
}

impl Glibc {
  /// Test if the provide key will be accepted by environment write operations.
  ///
  /// A name is valid if it does not trigger an `EINVAL` error when calling
  /// `setenv` or `clearenv`.
  /// A valid name has the following properties:
  /// - The name pointer is non-null; this is statically enforced by the `&str` type.
  /// - The name is not empty
  /// - The name does not contain the character `=`
  ///
  /// See <https://sourceware.org/git/?p=glibc.git;a=blob;f=stdlib/setenv.c;h=2b8b6cafa003236db5d4b44c1188e6992ed0c4ed;hb=HEAD#l288>.
  ///
  /// Note that read operations don't enforce these checks. For example, glibc
  /// allows to query a value for the empty environment variable name.
  pub fn validate_key_for_write(key: &CStr) -> Result<&CStr, InvalidKeyError> {
    if key.is_empty() {
      return Err(InvalidKeyError::Empty)
    };
    // TODO: use `key.bytes()` iterator once stable (feature "cstr_bytes")
    for (i, byte) in key.to_bytes().iter().copied().enumerate() {
      if byte == b'=' {
        return Err(InvalidKeyError::EqualSign(i))
      }
    }
    Ok(key)
  }
}
