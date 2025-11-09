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
