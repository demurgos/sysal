use crate::api::get_env::{GetEnv, GetEnvError};
use crate::api::unset_env::{UnsetEnv, UnsetEnvError};
use crate::mode::Mode;
use crate::mode::glibc::Glibc;
use crate::mode::w32::W32;
use std::ffi::{CStr, CString};
use crate::api::set_env::{SetEnv, SetEnvError};

pub struct VecEnv<Semantics>
where
  Semantics: ?Sized + Mode,
{
  variables: Vec<Semantics::Variable>,
}

impl<Semantics> VecEnv<Semantics>
where
  Semantics: ?Sized + Mode,
{
  pub fn new(variables: Vec<Semantics::Variable>) -> Self {
    Self { variables }
  }

  pub fn into_inner(self) -> Vec<Semantics::Variable> {
    let Self { variables } = self;
    variables
  }
}

impl<'env> GetEnv<&'_ CStr> for &'env VecEnv<Glibc> {
  type Value = &'env CStr;

  fn get_env(self, key: &'_ CStr) -> Result<Option<Self::Value>, GetEnvError> {
    let key_bytes = key.to_bytes();
    // based on <https://sourceware.org/git/?p=glibc.git;a=blob;f=stdlib/getenv.c;h=1a7b0bfc063e5fa63cd8d4bf4b661c09c621a0fb;hb=HEAD>
    for var in &self.variables {
      let var_bytes = var.to_bytes_with_nul();
      let Some(sep_and_val_bytes) = var_bytes.strip_prefix(key_bytes) else {
        continue;
      };
      let Some(val_bytes) = sep_and_val_bytes.strip_prefix(b"=") else {
        continue;
      };
      return Ok(match CStr::from_bytes_with_nul(val_bytes) {
        Ok(val) => Some(val),
        Err(e) => unreachable!("{e:?}"),
      });
    }
    Ok(None)
  }
}

impl<'env> UnsetEnv<&'_ CStr> for &'env mut VecEnv<Glibc> {
  fn unset_env(self, key: &'_ CStr) -> Result<(), UnsetEnvError> {
    let key = match Glibc::validate_key_for_write(key) {
      Ok(key) => key,
      Err(_e) => {
        return Err(UnsetEnvError::InvalidKey);
      }
    };
    let key_bytes = key.to_bytes();
    // let old_len = self.variables.len();
    // based on <https://sourceware.org/git?p=glibc.git;a=blob;f=stdlib/setenv.c;h=2b8b6cafa003236db5d4b44c1188e6992ed0c4ed;hb=HEAD#l298>
    self.variables.retain(|var| -> bool {
      match var.to_bytes_with_nul().strip_prefix(key_bytes) {
        Some(sep_and_val_bytes) if sep_and_val_bytes.starts_with(b"=") => false,
        _ => true,
      }
    });
    // let modified = self.variables.len() != old_len;
    Ok(())
  }
}

impl<'env> SetEnv<CString, CString> for &'env mut VecEnv<Glibc> {
  fn set_env(self, key: CString, value: CString) -> Result<(), SetEnvError> {
    let key = match Glibc::validate_key_for_write(key.as_c_str()) {
      Ok(key) => key,
      Err(_e) => {
        return Err(SetEnvError::InvalidKey);
      }
    };
    let key_bytes = key.to_bytes();
    let mut was_written = false;
    self.variables.retain_mut(|var| -> bool {
      let var_bytes = var.to_bytes_with_nul();
      match var_bytes.strip_prefix(key_bytes) {
        Some(sep_and_val_bytes) => {
          if was_written {
            return false;
          }

          match sep_and_val_bytes.strip_prefix(b"=") {
            Some(val_bytes)  => {
              let prefix_len = var_bytes.len() - val_bytes.len();
              let mut new_var = var_bytes[..prefix_len].to_vec();
              new_var.extend_from_slice(value.to_bytes_with_nul());
              let new_var = CString::from_vec_with_nul(new_var).unwrap(); // todo
              *var = new_var;
              was_written = true;
              return true;
            }
            None => return true,
          }
        },
        None => return true,
      }
    });
    Ok(())
  }
}

impl<'env> GetEnv<&'_ CStr> for &'env VecEnv<W32> {
  type Value = &'env CStr;

  fn get_env(self, key: &'_ CStr) -> Result<Option<Self::Value>, GetEnvError> {
    let key_bytes = key.to_bytes();
    for var in &self.variables {
      let var_bytes = var.to_bytes_with_nul();
      let Some(sep_and_val_bytes) = strip_prefix_ignore_ascii_case(var_bytes, key_bytes) else {
        continue;
      };
      let Some(val_bytes) = sep_and_val_bytes.strip_prefix(b"=") else {
        continue;
      };
      return Ok(match CStr::from_bytes_with_nul(val_bytes) {
        Ok(val) => Some(val),
        Err(e) => unreachable!("{e:?}"),
      });
    }
    Ok(None)
  }
}

fn strip_prefix_ignore_ascii_case<'s>(s: &'s [u8], prefix: &[u8]) -> Option<&'s [u8]> {
  let Some((head, tail)) = s.split_at_checked(prefix.len()) else {
    return None;
  };
  if head.eq_ignore_ascii_case(prefix) {
    Some(tail)
  } else {
    None
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn glibc_get() {
    let env = VecEnv::<Glibc>::new(vec![
      CString::new("foo=bar=qux").expect("test variable is valid"),
      CString::new("=foo=bar").expect("test variable is valid"),
    ]);
    assert_eq!(env.get_env(c"foo"), Ok(Some(c"bar=qux")));
    assert_eq!(env.get_env(c"foo=bar"), Ok(Some(c"qux")));
    assert_eq!(env.get_env(c"=foo"), Ok(Some(c"bar")));
    assert_eq!(env.get_env(c""), Ok(Some(c"foo=bar")));
    assert_eq!(env.get_env(c"unknown"), Ok(None));
    assert_eq!(env.get_env(c"FOO"), Ok(None));
  }

  #[test]
  fn glibc_unset() {
    let mut env = VecEnv::<Glibc>::new(vec![
      CString::new("foo=1").expect("test variable is valid"),
      CString::new("bar=2").expect("test variable is valid"),
    ]);
    assert_eq!(env.get_env(c"foo"), Ok(Some(c"1")));
    assert_eq!(env.get_env(c"bar"), Ok(Some(c"2")));
    assert_eq!(env.unset_env(c"foo"), Ok(()));
    assert_eq!(env.unset_env(c"foo"), Ok(()));
    assert_eq!(env.unset_env(c"unknown"), Ok(()));
    assert_eq!(env.get_env(c"foo"), Ok(None));
    assert_eq!(env.get_env(c"bar"), Ok(Some(c"2")));
  }

  #[test]
  fn glibc_set() {
    let mut env = VecEnv::<Glibc>::new(vec![
      CString::new("foo=1").expect("test variable is valid"),
      CString::new("bar=2").expect("test variable is valid"),
    ]);
    assert_eq!(env.set_env(CString::from(c"foo"), CString::from(c"3")), Ok(()));
    assert_eq!(env.get_env(c"foo"), Ok(Some(c"3")));
    assert_eq!(env.get_env(c"bar"), Ok(Some(c"2")));
  }

  #[test]
  fn w32_get() {
    let env = VecEnv::<W32>::new(vec![
      CString::new("foo=bar=qux").expect("test variable is valid"),
      CString::new("=foo=bar").expect("test variable is valid"),
    ]);
    assert_eq!(env.get_env(c"foo"), Ok(Some(c"bar=qux")));
    assert_eq!(env.get_env(c"foo=bar"), Ok(Some(c"qux")));
    assert_eq!(env.get_env(c"=foo"), Ok(Some(c"bar")));
    assert_eq!(env.get_env(c""), Ok(Some(c"foo=bar")));
    assert_eq!(env.get_env(c"unknown"), Ok(None));
    assert_eq!(env.get_env(c"FOO"), Ok(Some(c"bar=qux")));
  }
}
