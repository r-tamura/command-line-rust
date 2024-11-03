pub enum UniqrError {
    FileNotFoundError(std::io::Error, String),
    UnexpectedError(Box<dyn std::error::Error>),
}

impl std::fmt::Display for UniqrError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UniqrError::UnexpectedError(_) => write!(f, "処理中に不明なエラーが発生しました"),
            UniqrError::FileNotFoundError(e, filepath) => {
                write!(f, "{}: {}", filepath, e)
            }
        }
    }
}

impl std::error::Error for UniqrError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UniqrError::FileNotFoundError(e, _) => Some(e),
            UniqrError::UnexpectedError(e) => Some(e.as_ref()),
        }
    }
}

impl From<std::io::Error> for UniqrError {
    fn from(e: std::io::Error) -> Self {
        UniqrError::UnexpectedError(Box::new(e))
    }
}
