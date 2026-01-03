/// Trait representing a backend supporting environment variable write.
///
/// This abstract over [`setenv`](https://www.man7.org/linux/man-pages/man3/setenv.3.html)
/// with `overwrite = 1`.
pub trait SetEnv<K, V> {
  fn set_env(self, key: K, value: V) -> Result<(), SetEnvError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SetEnvError {
  #[error("failed to acquire exclusive access to the environment")]
  Acquire,
  #[error("invalid key provided")]
  InvalidKey,
}
