# SYStem Abstraction Layer for Rust

This repository contains Rust crates providing an abstraction layer for
operating system APIs such as environment variables, path or the filesystem.

The primary purpose of this library is to enable writing code independent of
the operating system of the build target. For example it allows to handle
Windows file paths even when building for Linux. As a comparison, the standard
library makes heavy use of conditional compilation to only expose APIs for
the target system.
