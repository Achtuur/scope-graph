use derive_more::{Display, From};

pub type ParseResult<T> = core::result::Result<T, ParseError>;

// uncomment this for test code
pub type ParseError = Box<dyn std::error::Error>;

// #[derive(Display, Debug, From)]
// pub enum Error {
    
// }

// impl std::error::Error for Error {}