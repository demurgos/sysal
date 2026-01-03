//! System abstraction layer

mod posix;
mod system;

pub trait System {
  type Path;
  type Env;
}
