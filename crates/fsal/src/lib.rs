//! File system abstraction layer

use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

pub mod tar;
// pub mod tar_gz;
// pub mod tar_xz;

pub enum ChildrenError {
  NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum ReadError {
  #[error("some path component does not exist")]
  NotFound,
  #[error("target entry is not a regular file")]
  NotFile,
  #[error("unexpected read error: {0}")]
  Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum ReadLinkError {
  #[error("some path component does not exist")]
  NotFound,
  #[error("target entry is not a symbolic link")]
  NotLink,
  #[error("unexpected read error: {0}")]
  Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum ReadDirError {
  #[error("some path component does not exist")]
  NotFound,
  #[error("unexpected read dir error: {0}")]
  Other(String),
}

pub trait FsxDirEntry<Path> {
  fn path(&self) -> &Path;
  fn file_type(&self) -> FileType;
}

/// Minimal directory entry
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimpleDirEntry<PathBuf> {
  pub path: PathBuf,
  pub file_type: FileType,
}

impl<Path> FsxDirEntry<Path> for SimpleDirEntry<Path> {
  fn path(&self) -> &Path {
    &self.path
  }

  fn file_type(&self) -> FileType {
    self.file_type
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum FileType {
  File,
  Dir,
  Symlink,
  Other,
}

// trait FsxPath<Segment>: IntoIterator<Item = Segment> {}
//
// trait FsxPathBuf<Segment> {
//   fn push(&mut self, segment: Segment);
// }

/// Read-only Abstract File System
pub trait ReadFsx<Path: ?Sized> {
  /// Owned type representing a path
  type PathBuf: AsRef<Path>;
  /// Owned directory entry
  type DirEntry: FsxDirEntry<Self::PathBuf>;

  fn read_file<P: AsRef<Path> + Send + Sync>(&self, path: P) -> impl Future<Output = Result<FsxFile, ReadError>> + Send;
  fn read<P: AsRef<Path> + Send + Sync>(&self, path: P) -> impl Future<Output = Result<Vec<u8>, ReadError>> + Send;
  fn read_link<P: AsRef<Path> + Send + Sync>(&self, path: P) -> impl Future<Output = Result<Self::PathBuf, ReadLinkError>> + Send;
  fn read_dir<P: AsRef<Path> + Send + Sync>(&self, path: P) -> impl Future<Output = Result<Vec<Self::DirEntry>, ReadDirError>> + Send;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FsxFile {
  /// Primary file content
  pub data: Vec<u8>,
  /// If present, Linux file mode
  ///
  /// TODO: Rename to POSIX file mode?
  pub linux_mode: Option<u32>,
}

/// A `ReadFsx` subtree granting access to only a given prefix.
pub struct SubTree<Tree, PathBuf> {
  pub tree: Tree,
  pub prefix: PathBuf,
}

// #[async_trait]
// impl<Path, Tree> ReadFsx<Path> for SubTree<Tree, Tree::PathBuf>
// where
//   Path: ?Sized,
//   Tree: ReadFsx<Path>,
//   Self: Send + Sync,
// {
//   type PathBuf = Tree::PathBuf;
//   type DirEntry = Tree::DirEntry;
//
//   async fn read<P: AsRef<Path> + Send + Sync>(&self, path: P) -> Result<Vec<u8>, ReadError> {
//     todo!()
//   }
//
//   async fn read_link<P: AsRef<Path> + Send + Sync>(&self, path: P) -> Result<Self::PathBuf, ReadLinkError> {
//     todo!()
//   }
//
//   async fn read_dir<P: AsRef<Path> + Send + Sync>(&self, path: P) -> Result<Vec<Self::DirEntry>, ReadDirError> {
//     todo!()
//   }
// }

impl<Tree> ReadFsx<[String]> for SubTree<Tree, Tree::PathBuf>
where
  Tree: ReadFsx<[String], PathBuf = Vec<String>, DirEntry = SimpleDirEntry<Vec<String>>> + Send + Sync,
  Self: Send + Sync,
{
  type PathBuf = Tree::PathBuf;
  type DirEntry = Tree::DirEntry;

  async fn read<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<Vec<u8>, ReadError> {
    let mut full_path = self.prefix.clone();
    full_path.extend_from_slice(path.as_ref());
    self.tree.read(full_path).await
  }

  async fn read_file<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<FsxFile, ReadError> {
    let mut full_path = self.prefix.clone();
    full_path.extend_from_slice(path.as_ref());
    self.tree.read_file(full_path).await
  }

  async fn read_link<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<Self::PathBuf, ReadLinkError> {
    let mut full_path = self.prefix.clone();
    full_path.extend_from_slice(path.as_ref());
    let pointee = self.tree.read_link(full_path).await?;
    assert!(
      pointee.first().map(|component| !component.is_empty()).unwrap_or(true),
      "pointee can't be absolute"
    );
    Ok(pointee)
  }

  async fn read_dir<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<Vec<Self::DirEntry>, ReadDirError> {
    let mut full_path = self.prefix.clone();
    full_path.extend_from_slice(path.as_ref());
    let entries: Vec<_> = self.tree.read_dir(full_path).await?;
    let entries: Vec<_> = entries
      .into_iter()
      .map(|mut e| {
        assert!(e.path.len() >= self.prefix.len());
        let prefix = e.path.drain(0..self.prefix.len());
        for (i, component) in prefix.enumerate() {
          assert_eq!(component, self.prefix[i]);
        }
        e
      })
      .collect();
    Ok(entries)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
enum ComponentError {
  #[error("found non-normal component: {0}")]
  NotNormal(String),
  #[error("non-utf8 component")]
  Utf8,
}

trait PathExt {
  fn eq_utf8_segments(&self, segments: impl Iterator<Item = String>) -> Result<bool, ComponentError>;
}

impl<P> PathExt for P
where
  P: AsRef<Path>,
{
  fn eq_utf8_segments(&self, segments: impl Iterator<Item = String>) -> Result<bool, ComponentError> {
    let path = self.as_ref();
    let mut components = path.components().map(|c| match c {
      Component::Normal(c) => c.to_str().ok_or(ComponentError::Utf8),
      c => Err(ComponentError::NotNormal(format!("{c:?}"))),
    });
    for segment in segments {
      match components.next().transpose()? {
        Some(c) if c == segment => continue,
        _ => return Ok(false),
      }
    }
    Ok(components.next().transpose()?.is_none())
  }
}

// TODO: Provide implementation based on `cap-std`

#[derive(Debug, Clone)]
pub struct LocalFs {
  start: PathBuf,
}

impl LocalFs {
  pub fn new(base: PathBuf) -> Self {
    Self { start: base }
  }
}

impl ReadFsx<Path> for LocalFs {
  type PathBuf = PathBuf;
  type DirEntry = SimpleDirEntry<Self::PathBuf>;

  async fn read<P: AsRef<Path> + Send + Sync>(&self, path: P) -> Result<Vec<u8>, ReadError> {
    match std::fs::read(self.start.join(path)) {
      Ok(data) => Ok(data),
      Err(e) if e.kind() == io::ErrorKind::NotFound => Err(ReadError::NotFound),
      Err(e) => Err(ReadError::Other(e.to_string())),
    }
  }

  async fn read_file<P: AsRef<Path> + Send + Sync>(&self, path: P) -> Result<FsxFile, ReadError> {
    let path = path.as_ref();
    match std::fs::read(self.start.join(path)) {
      Ok(data) => {
        let meta = std::fs::metadata(self.start.join(path)).map_err(|e| ReadError::Other(e.to_string()))?;
        let linux_mode = meta.mode();
        Ok(FsxFile {
          data,
          linux_mode: Some(linux_mode),
        })
      }
      Err(e) if e.kind() == io::ErrorKind::NotFound => Err(ReadError::NotFound),
      Err(e) => Err(ReadError::Other(e.to_string())),
    }
  }

  async fn read_link<P: AsRef<Path> + Send + Sync>(&self, _path: P) -> Result<Self::PathBuf, ReadLinkError> {
    todo!()
  }

  async fn read_dir<P: AsRef<Path> + Send + Sync>(&self, _path: P) -> Result<Vec<Self::DirEntry>, ReadDirError> {
    let _ = 0; // Workaround for <https://github.com/rust-lang/rust-clippy/issues/10243>
    todo!()
  }
}

impl ReadFsx<[String]> for LocalFs {
  type PathBuf = Vec<String>;
  type DirEntry = SimpleDirEntry<Self::PathBuf>;

  async fn read<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<Vec<u8>, ReadError> {
    let mut p = self.start.clone();
    for segment in path.as_ref() {
      p.push(segment)
    }

    match std::fs::read(&p) {
      Ok(data) => Ok(data),
      Err(e) if e.kind() == io::ErrorKind::NotFound => Err(ReadError::NotFound),
      Err(e) => Err(ReadError::Other(e.to_string())),
    }
  }

  async fn read_file<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<FsxFile, ReadError> {
    let mut p = self.start.clone();
    for segment in path.as_ref() {
      p.push(segment)
    }

    match std::fs::read(&p) {
      Ok(data) => {
        let meta = std::fs::metadata(&p).map_err(|e| ReadError::Other(e.to_string()))?;
        let linux_mode = meta.mode();
        Ok(FsxFile {
          data,
          linux_mode: Some(linux_mode),
        })
      }
      Err(e) if e.kind() == io::ErrorKind::NotFound => Err(ReadError::NotFound),
      Err(e) => Err(ReadError::Other(e.to_string())),
    }
  }

  async fn read_link<P: AsRef<[String]> + Send + Sync>(&self, _path: P) -> Result<Self::PathBuf, ReadLinkError> {
    todo!()
  }

  async fn read_dir<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<Vec<Self::DirEntry>, ReadDirError> {
    let mut base: Vec<String> = Vec::new();
    let mut p = self.start.clone();
    for segment in path.as_ref() {
      base.push(segment.to_string());
      p.push(segment)
    }
    let read_dir = match std::fs::read_dir(&p) {
      Ok(read_dir) => read_dir,
      Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(ReadDirError::NotFound),
      Err(e) => return Err(ReadDirError::Other(e.to_string())),
    };
    let mut entries = Vec::new();
    for entry in read_dir {
      let entry = entry.unwrap();
      let segment = entry.file_name().to_str().unwrap().to_string();
      let mut path_buf = base.clone();
      let meta = entry.metadata().unwrap();
      let file_type = if meta.is_symlink() {
        FileType::Symlink
      } else if meta.is_dir() {
        FileType::Dir
      } else if meta.is_file() {
        FileType::File
      } else {
        FileType::Other
      };
      path_buf.push(segment);
      entries.push(SimpleDirEntry {
        path: path_buf,
        file_type,
      })
    }
    Ok(entries)
  }
}
