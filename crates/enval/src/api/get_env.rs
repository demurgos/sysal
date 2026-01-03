/// Trait representing a backend supporting environment variable retrievable.
/// 
/// This abstract over [`getenv`](https://www.man7.org/linux/man-pages/man3/getenv.3.html)
pub trait GetEnv<K> {
  type Value;

  fn get_env(self, key: K) -> Result<Option<Self::Value>, GetEnvError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GetEnvError {}
