use core::fmt;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::io::{Cursor, Read};
use std::path::Component;
use std::sync::Mutex;
use ::tar::EntryType;
use tar::Archive;
use crate::{ComponentError, FileType, FsxFile, PathExt, ReadDirError, ReadError, ReadFsx, ReadLinkError, SimpleDirEntry};

/// File System stored in a _tar_ archive.
pub struct Tar {
  /// Tar content (decompression occurs in the constructor)
  archive: Archive<Cursor<Vec<u8>>>,
  /// The `Tar` tree auto-detects if there's a singe directory at the root
  pub(crate) root_dir: Option<String>,
}

/// File System stored in a _tar_ archive.
// #[derive(Clone, PartialEq, Eq)]
pub struct TarFs<'a> {
  /// Tar content (decompression occurs in the constructor)
  entries: std::sync::Mutex<Vec<::tar::Entry<'a, Cursor<Vec<u8>>>>>,
  /// The `Tar` tree auto-detects if there's a singe directory at the root
  pub(crate) root_dir: Option<String>,
}

impl Debug for Tar {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct(core::any::type_name::<Self>())
      .field("archive", &"...")
      .field("root_dir", &self.root_dir)
      .finish()
  }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum TarError {
  #[error("io error: {0}")]
  Io(String),
  #[error("invalid entry name")]
  InvalidEntryName,
}

impl Tar {
  pub fn new(tar: Vec<u8>) -> Result<Self, TarError> {
    let tar = Cursor::new(tar);

    let mut archive = tar::Archive::new(tar.clone());
    let mut root_dir = None;
    let mut has_multiple_entries_at_root = false;
    let mut has_multiple_entries = false;
    let entries = archive.entries_with_seek().map_err(|e| TarError::Io(e.to_string()))?;
    for e in entries {
      let e = e.map_err(|e| TarError::Io(e.to_string()))?;

      if e.header().entry_type() == EntryType::XGlobalHeader {
        // see `read_dir` for detailed explanation. The short version is that
        // this entry contains global metadata and should be ignored
        // `git-archive` uses it to add an attribute named `comment` with the
        // commit oid.
        continue;
      }

      let p = e.path().map_err(|e| TarError::Io(e.to_string()))?;
      match p.components().next() {
        Some(Component::Normal(segment)) => {
          let segment = segment.to_str().ok_or(TarError::InvalidEntryName)?;
          match root_dir.as_deref() {
            Some(r) => {
              has_multiple_entries = true;
              if r != segment {
                has_multiple_entries_at_root = true;
              }
            }
            None => root_dir = Some(segment.to_string()),
          }
        }
        _ => has_multiple_entries_at_root = true,
      }
    }
    if has_multiple_entries_at_root || !has_multiple_entries {
      root_dir = None;
    }

    let mut archive = tar::Archive::new(tar.clone());

    Ok(Self { archive, root_dir })
  }

  pub fn new_fs<'a>(&'a self) -> TarFs<'a> {

    let entries = self.archive
      .entries_with_seek()
      .map_err(|e| ReadError::Other(e.to_string())).unwrap();

    let entries = Vec::from_iter(entries.map(|e| e.map_err(|e| ReadError::Other(e.to_string())).unwrap()));

    TarFs { entries: Mutex::new(entries), root_dir: self.root_dir.clone() }
  }
}

impl ReadFsx<[String]> for TarFs<'_> {
  type PathBuf = Vec<String>;
  type DirEntry = SimpleDirEntry<Self::PathBuf>;

  async fn read<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<Vec<u8>, ReadError> {
    let path = path.as_ref();
    let path = self.root_dir.as_ref().into_iter().chain(path.iter()).cloned();

    let entries = self.entries.iter();
    for e in entries {
      let p = e.path().map_err(|e| ReadError::Other(e.to_string()))?;
      let is_wanted_entry = p
        .eq_utf8_segments(path.clone())
        .map_err(|e| ReadError::Other(e.to_string()))?;
      if is_wanted_entry {
        if !matches!(e.header().entry_type(), EntryType::Regular) {
          return Err(ReadError::NotFile);
        }
        let mut body: Vec<u8> = Vec::new();
        // e.read_to_end(&mut body).map_err(|e| ReadError::Other(e.to_string()))?;
        return Ok(body);
      }
    }
    Err(ReadError::NotFound)
  }

  async fn read_file<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<FsxFile, ReadError> {
    let path = path.as_ref();
    let path = self.root_dir.as_ref().into_iter().chain(path.iter()).cloned();

    let entries = self.entries.iter();
    for e in entries {
      let p = e.path().map_err(|e| ReadError::Other(e.to_string()))?;
      let is_wanted_entry = p
        .eq_utf8_segments(path.clone())
        .map_err(|e| ReadError::Other(e.to_string()))?;
      if is_wanted_entry {
        if !matches!(e.header().entry_type(), EntryType::Regular) {
          return Err(ReadError::NotFile);
        }
        let mut body: Vec<u8> = Vec::new();
        let header = e.header();
        let linux_file_mode = header.mode().map_err(|e| ReadError::Other(e.to_string()))?;
        // e.read_to_end(&mut body).map_err(|e| ReadError::Other(e.to_string()))?;
        return Ok(FsxFile {
          data: body,
          linux_mode: Some(linux_file_mode),
        });
      }
    }
    Err(ReadError::NotFound)
  }

  async fn read_link<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<Self::PathBuf, ReadLinkError> {
    let path = path.as_ref();
    let path = self.root_dir.as_ref().into_iter().chain(path.iter()).cloned();

    let entries = self.entries.iter();
    for e in entries {
      let p = e.path().map_err(|e| ReadLinkError::Other(e.to_string()))?;
      let is_wanted_entry = p
        .eq_utf8_segments(path.clone())
        .map_err(|e| ReadLinkError::Other(e.to_string()))?;
      if is_wanted_entry {
        if !matches!(e.header().entry_type(), EntryType::Symlink) {
          return Err(ReadLinkError::NotLink);
        }
        let target = e
          .header()
          .link_name()
          .map_err(|e| ReadLinkError::Other(e.to_string()))?
          .ok_or_else(|| ReadLinkError::Other("no link target".to_string()))?;
        return Ok(
          target
            .to_str()
            .ok_or_else(|| ReadLinkError::Other("non utf8 link target".to_string()))?
            .split('/')
            .map(|component| component.to_string())
            .collect(),
        );
      }
    }
    Err(ReadLinkError::NotFound)
  }

  async fn read_dir<P: AsRef<[String]> + Send + Sync>(&self, path: P) -> Result<Vec<Self::DirEntry>, ReadDirError> {
    let base_path = path.as_ref();
    let path = self.root_dir.as_ref().into_iter().chain(base_path.iter()).cloned();
    let is_empty_path = path.clone().next().is_none();

    let entries = self.entries.iter();

    let mut result: Vec<SimpleDirEntry<Self::PathBuf>> = Vec::new();
    // A directory is found if an entry with the directory name is found
    // An exception is if the root is the empty path
    let mut found_self = is_empty_path;

    // Directories can be explicit or implicit.
    // Implicit directories are created whenever a file entry is found while the directory itself is missing.
    let mut directories = BTreeSet::new();
    'entries: for e in entries {
      let entry_path = e.path().map_err(|e| ReadDirError::Other(e.to_string()))?;
      let mut entry_components = entry_path.components().map(|c| match c {
        Component::Normal(c) => c.to_str().ok_or(ComponentError::Utf8),
        c => Err(ComponentError::NotNormal(format!("{c:?}"))),
      });
      for main_component in path.clone() {
        let entry_component = entry_components
          .next()
          .transpose()
          .map_err(|e| ReadDirError::Other(e.to_string()))?;
        if entry_component != Some(main_component.as_str()) {
          continue 'entries;
        }
      }
      // At this point `path` is a prefix or equal to `entry_path`
      let entry_name = entry_components
        .next()
        .transpose()
        .map_err(|e| ReadDirError::Other(e.to_string()))?;
      let child_path = match entry_name {
        None => {
          // The entry is the base path itself
          found_self = true;
          continue 'entries;
        }
        Some(entry) => {
          let mut child_path = Vec::new();
          child_path.extend(base_path.iter().cloned());
          child_path.push(entry.to_string());
          child_path
        }
      };
      if entry_components.next().is_some() {
        directories.insert(child_path);
        continue 'entries; // There are some deeper components, this indicates a nested directory
      }
      let file_type = match e.header().entry_type() {
        EntryType::Regular => FileType::File,
        EntryType::Symlink => FileType::Symlink,
        EntryType::Directory => {
          let has_trailing_slash = entry_path.to_str().map(|p| p.ends_with('/')).unwrap_or(false);
          debug_assert!(has_trailing_slash);
          directories.insert(child_path);
          continue 'entries; // There are some deeper components, this indicates a nested directory
        }
        EntryType::XGlobalHeader => {
          // pax global header, defines a key-value list of attributes that
          // should be applied to all following files
          // We don't handle such metadata at all, so we skip this
          //
          // This usually occurs for files created with `git-archive` and contains
          // the optional attributes `comment` with the commit object id (sha1)
          // or `mtime` with a dedicated modification time
          // See <https://github.com/git/git/blob/e66fd72e972df760a53c3d6da023c17adfc426d6/archive-tar.c#L332>
          //
          // I tried create a similar archive with `tar` and the following command
          // but I did not succeed:
          // ```
          // tar --create --gzip --verbose --file ./pax_global_header.tar.gz --format=pax --pax-option=globexthdr.comment:=6304632eaa2107bb1763d29e213ff166ff6104c0 ./pax_global_header
          // ```
          // let extensions: Option<PaxExtensions> = e.pax_extensions().map_err(|e| {
          //   ReadDirError::Other(format!(
          //     "failed to read pax extensions for XGlobalHeader entry {child_path:?}"
          //   ))
          // })?;
          // if let Some(extensions) = extensions {
          //   for (ext_index, ext) in extensions.enumerate() {
          //     let ext: PaxExtension = ext.map_err(|e| {
          //       ReadDirError::Other(format!(
          //         "failed to read pax extension value for XGlobalHeader entry {child_path:?} at index {ext_index}"
          //       ))
          //     })?;
          //     dbg!(ext.key());
          //     dbg!(ext.value());
          //   }
          // }
          continue 'entries;
        }
        _ => FileType::Other,
      };
      result.push(SimpleDirEntry {
        path: child_path,
        file_type,
      });
    }
    for path in directories {
      result.push(SimpleDirEntry {
        path,
        file_type: FileType::Dir,
      });
    }
    if result.is_empty() && !found_self {
      return Err(ReadDirError::NotFound);
    }
    Ok(result)
  }
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeSet;
  use crate::{FsxDirEntry};
  use super::*;

  async fn list_files<T: ReadFsx<[String]>>(tree: &T) -> Vec<String> {
    let mut stack: Vec<T::DirEntry> = tree.read_dir([]).await.unwrap();
    let mut paths: BTreeSet<String> = BTreeSet::new();

    while let Some(entry) = stack.pop() {
      let path: &[String] = entry.path().as_ref();
      let is_fresh = paths.insert(path.join("/"));
      if !is_fresh {
        continue;
      }
    }

    Vec::from_iter(paths.into_iter())
  }

  #[tokio::test]
  async fn read_tgz() {
    // archive created with `tar -czvf archive.tar.gz archive`
    let bytes = include_bytes!("../../../llvm-20.1.8.src.tar");
    let tree = Tar::new(bytes.to_vec()).unwrap();
    let files = list_files(&tree).await;
    assert_eq!(files.len(), 100);
  }
}
