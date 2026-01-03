/// Trait representing a backend supporting environment variable removal.
///
/// This abstract over [`unsetenv`](https://www.man7.org/linux/man-pages/man3/unsetenv.3.html)
pub trait UnsetEnv<K> {
  /// Remove the environment variable for the provided key.
  ///
  /// Returns a boolean indicating if the key was present.
  fn unset_env(self, key: K) -> Result<(), UnsetEnvError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UnsetEnvError {
  #[error("failed to acquire exclusive access to the environment")]
  Acquire,
  #[error("invalid key provided")]
  InvalidKey,
}
