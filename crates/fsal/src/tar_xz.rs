use core::fmt;
use display_full_error::DisplayFullErrorExt;
use crate::{FsxFile, ReadDirError, ReadError, ReadFsx, ReadLinkError, SimpleDirEntry};
use crate::tar::{Tar, TarError};

#[derive(Clone, PartialEq, Eq)]
pub struct TarXz {
  /// Decompressed tar content (decompression occurs in the constructor)
  tar: Tar,
}

impl fmt::Debug for TarXz {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct(core::any::type_name::<Self>())
      .field("archive", &"...")
      .field("root_dir", &self.tar.root_dir)
      .finish()
  }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum TarXzError {
  #[error("decompression error: {0}")]
  Decompress(String),
  #[error("io error: {0}")]
  Io(String),
  #[error("invalid entry name")]
  InvalidEntryName,
}

impl TarXz {
  pub fn new(mut txz: &[u8]) -> Result<Self, TarXzError> {
    let mut tar: Vec<u8> = Vec::new();
    match lzma_rs::xz_decompress(&mut txz, &mut tar) {
      Err(e) => Err(TarXzError::Decompress(e.to_string_full())),
      Ok(()) => {
        match Tar::new(tar) {
          Err(TarError::Io(err)) => Err(TarXzError::Io(err)),
          Err(TarError::InvalidEntryName) => Err(TarXzError::InvalidEntryName),
          Ok(tar) => Ok(Self { tar }),
        }
      },
    }
  }
}

impl ReadFsx<[String]> for TarXz {
  type PathBuf = Vec<String>;
  type DirEntry = SimpleDirEntry<Self::PathBuf>;

  async fn read<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<Vec<u8>, ReadError> {
    self.tar.read(path).await
  }

  async fn read_file<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<FsxFile, ReadError> {
    self.tar.read_file(path).await
  }

  async fn read_link<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<Self::PathBuf, ReadLinkError> {
    self.tar.read_link(path).await
  }

  async fn read_dir<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<Vec<Self::DirEntry>, ReadDirError> {
    self.tar.read_dir(path).await
  }
}

// #[cfg(test)]
// mod tests {
//   use std::collections::BTreeSet;
//   use itertools::Itertools;
//   use crate::{FileType, FsxDirEntry, SubTree};
//   use super::*;
//
//   async fn list_files<T: ReadFsx<[String]>>(tree: &T) -> Vec<String> {
//     let mut stack: Vec<T::DirEntry> = tree.read_dir([String::from("/")]).await.unwrap();
//     let mut paths: BTreeSet<String> = BTreeSet::new();
//
//     while let Some(entry) = stack.pop() {
//       let path: &[String] = entry.path().as_ref();
//       let is_fresh = paths.insert(path.join("/"));
//       if !is_fresh {
//         continue;
//       }
//     }
//
//     Vec::from_iter(paths.into_iter())
//   }
//
//   #[tokio::test]
//   async fn read_tgz() {
//     // archive created with `tar -czvf archive.tar.gz archive`
//     let bytes = include_bytes!("../../../llvm-20.1.8.src.tar.xz");
//     let tree = TarXz::new(bytes).unwrap();
//     {
//       let actual = tree.read(["foo.txt".to_string()]).await.unwrap();
//       let expected = b"Hello, World!\n".to_vec();
//       assert_eq!(actual, expected);
//     }
//     {
//       let actual = tree.read_link(["foolink.txt".to_string()]).await.unwrap();
//       let expected = vec![".".to_string(), "foo.txt".to_string()];
//       assert_eq!(actual, expected);
//     }
//     {
//       let actual = tree.read(["bar".to_string(), "baz.txt".to_string()]).await.unwrap();
//       let expected = b"bar/baz\n".to_vec();
//       assert_eq!(actual, expected);
//     }
//     {
//       let actual = tree.read_dir([]).await.unwrap();
//       let expected = vec![
//         SimpleDirEntry {
//           path: vec!["foo.txt".to_string()],
//           file_type: FileType::File,
//         },
//         SimpleDirEntry {
//           path: vec!["foolink.txt".to_string()],
//           file_type: FileType::Symlink,
//         },
//         SimpleDirEntry {
//           path: vec!["bar".to_string()],
//           file_type: FileType::Dir,
//         },
//         SimpleDirEntry {
//           path: vec!["empty".to_string()],
//           file_type: FileType::Dir,
//         },
//       ];
//       assert_eq!(actual, expected);
//     }
//     {
//       let actual = tree.read_dir(["bar".to_string()]).await.unwrap();
//       let expected = vec![
//         SimpleDirEntry {
//           path: vec!["bar".to_string(), "baz.txt".to_string()],
//           file_type: FileType::File,
//         },
//         SimpleDirEntry {
//           path: vec!["bar".to_string(), "empty.txt".to_string()],
//           file_type: FileType::File,
//         },
//       ];
//       assert_eq!(actual, expected);
//     }
//     {
//       let actual = tree.read_dir(["empty".to_string()]).await.unwrap();
//       let expected = vec![];
//       assert_eq!(actual, expected);
//     }
//     let sub_tree = SubTree {
//       tree,
//       prefix: vec!["bar".to_string()],
//     };
//     {
//       let actual = sub_tree.read(["baz.txt".to_string()]).await.unwrap();
//       let expected = b"bar/baz\n".to_vec();
//       assert_eq!(actual, expected);
//     }
//   }
// }
