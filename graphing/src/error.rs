use derive_more::{Display, From};

pub type RenderResult<T> = core::result::Result<T, RenderError>;

#[derive(Display, Debug, From)]
pub enum RenderError {
    #[from]
    IoError(std::io::Error),

    #[from]
    StringConversionError(std::string::FromUtf8Error),
}

impl std::error::Error for RenderError {}
