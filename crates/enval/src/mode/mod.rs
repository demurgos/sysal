pub mod w32;
pub mod glibc;

pub trait Mode {
  /// The type for a variable string.
  /// 
  /// A variable string has the format `<key>=<value>`.
  type Variable;
}
