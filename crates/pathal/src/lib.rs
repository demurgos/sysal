//! Path abstraction layer

pub mod posix;
pub mod utf8;
pub mod std;

pub trait PathModel {
  /// Owned path buffer
  type Buf;
  /// Readonly view into an existing path buffer
  type View<'path>;

  /// Owned data describing how to navigate to a root node.
  ///
  /// For POSIX file systems, this is `()`.
  /// For Windows systems, this holds the network host or drive letter.
  type Root;
  /// Readonly view into a `Self::Root`.
  type RootView<'path>;
  /// Owned data describing how to navigate through a regular link.
  type Link;
  /// Readonly view into a `Self::Link`.
  type LinkView<'path>;
}

// pub struct AbsolutePath(pub PathBuf);

pub struct AnyPathBuf<TyModel: PathModel> {
  commands: Vec<AnyCommandBuf<TyModel>>,
}

pub enum AnyCommandBuf<TyModel: PathModel> {
  Current,
  Parent,
  Root(TyModel::Root),
  Link(TyModel::Link),
}

pub struct AnyPathView<'path, TyModel: PathModel> {
  commands: &'path [AnyCommandView<'path, TyModel>],
}

pub enum AnyCommandView<'path, TyModel: PathModel> {
  Current,
  Parent,
  Root(TyModel::RootView<'path>),
  Link(TyModel::LinkView<'path>),
}
