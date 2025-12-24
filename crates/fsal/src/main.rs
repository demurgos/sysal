use std::collections::BTreeSet;
use fsal::{FileType, ReadFsx};
use fsal::tar::Tar;
use fsal::{FsxDirEntry};

async fn list_files<T: ReadFsx<[String]>>(tree: &T) -> Vec<String> {
  let mut stack: Vec<T::DirEntry> = tree.read_dir([]).await.unwrap();
  let mut paths: BTreeSet<String> = BTreeSet::new();

  while let Some(entry) = stack.pop() {
    let path: &[String] = entry.path().as_ref();
    let is_fresh = paths.insert(path.join("/"));
    // dbg!((&path, paths.len(), is_fresh));
    if !is_fresh || paths.len() > 100 {
      continue;
    }
    match entry.file_type() {
      FileType::Dir => {
        stack.extend(tree.read_dir(path).await.unwrap());
      }
      FileType::File => {}
      FileType::Symlink => {}
      FileType::Other => {}
    }
  }

  Vec::from_iter(paths.into_iter())
}

#[tokio::main]
async fn main() {
  // archive created with `tar -czvf archive.tar.gz archive`
  let bytes = include_bytes!("../../../llvm-20.1.8.src.tar");
  let tree = Tar::new(bytes.to_vec()).unwrap();
  let files = list_files(&tree).await;
  assert_eq!(files.len(), 221);
}
