//! Environment variable abstraction layer

// todo: abstract the following Windows 32 APIs:
//       _environ _putenv _putenv_s _searchenv _searchenv_s _dupenv_s _wputenv
//       _wputenv_s _wsearchenv getenv getenv_s putenv _wdupenv_s _wenviron
//       _wgetenv _wgetenv_s _wsearchenv_s tzset

pub mod api;
pub mod backend;
pub mod mode;
