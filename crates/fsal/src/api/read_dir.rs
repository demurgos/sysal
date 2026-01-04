use pathal::PathModel;

pub trait ReadDirBlocking<TyPath: PathModel> {
  fn read_dir_blocking<'path>(self, path: TyPath::View<'path>) -> Result<Vec<TyPath::Buf>, ReadDirBlockingError>;
}

pub enum ReadDirBlockingError {}
