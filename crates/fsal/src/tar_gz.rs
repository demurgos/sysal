use core::fmt;
use std::io::{Read};
use crate::{FsxFile, ReadDirError, ReadError, ReadFsx, ReadLinkError, SimpleDirEntry};
use crate::tar::{Tar, TarError};

#[derive(Clone, PartialEq, Eq)]
pub struct TarGz {
  /// Dezipped tar content (decompression occurs in the constructor)
  tar: Tar,
}

impl fmt::Debug for TarGz {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct(core::any::type_name::<Self>())
      .field("archive", &"...")
      .field("root_dir", &self.tar.root_dir)
      .finish()
  }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum TarGzError {
  #[error("io error: {0}")]
  Io(String),
  #[error("invalid entry name")]
  InvalidEntryName,
}

impl TarGz {
  pub fn new(tgz: &[u8]) -> Result<Self, TarGzError> {
    let mut stream = flate2::read::GzDecoder::new(tgz);
    let mut tar: Vec<u8> = Vec::new();
    stream
      .read_to_end(&mut tar)
      .map_err(|e| TarGzError::Io(e.to_string()))?;
    match Tar::new(tar) {
      Ok(tar) => Ok(Self { tar }),
      Err(TarError::Io(err)) => Err(TarGzError::Io(err)),
      Err(TarError::InvalidEntryName) => Err(TarGzError::InvalidEntryName),
    }
  }
}

impl ReadFsx<[String]> for TarGz {
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

#[cfg(test)]
mod tests {
  use crate::{FileType, SubTree};
  use super::*;

  #[tokio::test]
  async fn read_tgz() {
    // archive created with `tar -czvf archive.tar.gz archive`
    let bytes = include_bytes!("../test-resources/targz/archive.tar.gz");
    let tree = TarGz::new(bytes).unwrap();
    {
      let actual = tree.read(["foo.txt".to_string()]).await.unwrap();
      let expected = b"Hello, World!\n".to_vec();
      assert_eq!(actual, expected);
    }
    {
      let actual = tree.read_link(["foolink.txt".to_string()]).await.unwrap();
      let expected = vec![".".to_string(), "foo.txt".to_string()];
      assert_eq!(actual, expected);
    }
    {
      let actual = tree.read(["bar".to_string(), "baz.txt".to_string()]).await.unwrap();
      let expected = b"bar/baz\n".to_vec();
      assert_eq!(actual, expected);
    }
    {
      let actual = tree.read_dir([]).await.unwrap();
      let expected = vec![
        SimpleDirEntry {
          path: vec!["foo.txt".to_string()],
          file_type: FileType::File,
        },
        SimpleDirEntry {
          path: vec!["foolink.txt".to_string()],
          file_type: FileType::Symlink,
        },
        SimpleDirEntry {
          path: vec!["bar".to_string()],
          file_type: FileType::Dir,
        },
        SimpleDirEntry {
          path: vec!["empty".to_string()],
          file_type: FileType::Dir,
        },
      ];
      assert_eq!(actual, expected);
    }
    {
      let actual = tree.read_dir(["bar".to_string()]).await.unwrap();
      let expected = vec![
        SimpleDirEntry {
          path: vec!["bar".to_string(), "baz.txt".to_string()],
          file_type: FileType::File,
        },
        SimpleDirEntry {
          path: vec!["bar".to_string(), "empty.txt".to_string()],
          file_type: FileType::File,
        },
      ];
      assert_eq!(actual, expected);
    }
    {
      let actual = tree.read_dir(["empty".to_string()]).await.unwrap();
      let expected = vec![];
      assert_eq!(actual, expected);
    }
    let sub_tree = SubTree {
      tree,
      prefix: vec!["bar".to_string()],
    };
    {
      let actual = sub_tree.read(["baz.txt".to_string()]).await.unwrap();
      let expected = b"bar/baz\n".to_vec();
      assert_eq!(actual, expected);
    }
  }
}
