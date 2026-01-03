use crate::api::get_env::{GetEnv, GetEnvError};

/// A simple environment implementation where all operations fail.
///
/// The main purpose of this implementation is testing error paths.
pub struct FailEnv;

impl<'env> GetEnv<&str> for &'env FailEnv {
  type Value = String;

  fn get_env(self, _key: &str) -> Result<Option<Self::Value>, GetEnvError> {
    Ok(None)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fail_env() {
    let res = FailEnv.get_env("test");
    assert!(res.is_err());
  }
}
