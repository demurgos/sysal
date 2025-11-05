//! Environment variable abstraction layer

#[derive(Debug, thiserror::Error)]
pub enum GetError {
  #[error("environment variable not found for the provided key")]
  NotFound,
}

pub trait EnvRead<K> {
  type Value;

  fn get(&self, key: K) -> Result<Self::Value, GetError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SetError<Inner> {
  #[error(transparent)]
  Other(Inner),
}

pub trait EnvWrite<K, V> {
  type SetError;

  fn set(&self, key: K, value: V) -> Result<(), SetError<Self::SetError>>;
}

pub struct FailEnv;

impl EnvRead<&str> for FailEnv {
  type Value = String;

  fn get(&self, _key: &str) -> Result<Self::Value, GetError> {
    Err(GetError::NotFound)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fail_env() {
    let res = FailEnv.get("test");
    assert!(res.is_err());
  }
}
