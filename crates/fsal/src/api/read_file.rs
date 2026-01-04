use pathal::PathModel;

pub trait ReadFileBlocking<TyPath: PathModel> {
  fn read_file_blocking<'path>(self, path: TyPath::View<'path>) -> Result<Vec<u8>, ReadFileBlockingError>;
}

pub enum ReadFileBlockingError {}
