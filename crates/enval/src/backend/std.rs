use crate::api::get_env::{GetEnv, GetEnvError};
use crate::api::set_env::{SetEnv, SetEnvError};
use crate::api::unset_env::{UnsetEnv, UnsetEnvError};
use std::ffi::{OsStr, OsString};

pub struct StdEnv;

impl<'env> GetEnv<&'_ OsStr> for &'env StdEnv {
  type Value = OsString;

  fn get_env(self, key: &'_ OsStr) -> Result<Option<Self::Value>, GetEnvError> {
    match std::env::var_os(key) {
      None => Ok(None),
      Some(val) => Ok(Some(val)),
    }
  }
}

impl<'env> UnsetEnv<&'_ OsStr> for &'env StdEnv {
  fn unset_env(self, key: &'_ OsStr) -> Result<(), UnsetEnvError> {
    match num_threads::is_single_threaded() {
      Some(true) => unsafe {
        std::env::remove_var(key);
        Ok(())
      },
      _ => Err(UnsetEnvError::Acquire),
    }
  }
}

impl<'env> SetEnv<OsString, OsString> for &'env StdEnv {
  fn set_env(self, key: OsString, value: OsString) -> Result<(), SetEnvError> {
    match num_threads::is_single_threaded() {
      Some(true) => unsafe {
        std::env::set_var(key, value);
        Ok(())
      },
      _ => Err(SetEnvError::Acquire),
    }
  }
}
