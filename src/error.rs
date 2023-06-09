use failure::{Error, Fail};

#[derive(Fail, Debug)]
pub enum MyError {
    #[fail(display = "Serde error : {}", _0)]
    Serde(#[cause] serde_json::Error),
    #[fail(display = "Serde error : {}", _0)]
    Write(#[cause] std::io::Error),
    #[fail(display = "Key not found")]
    KeyNotFound,
    #[fail(display = "Wrong command")]
    UnexpectedCommand,
    #[fail(display = "The argument is not formated correctly")]
    MalformatedArgument,
    #[fail(display = "The server was already using a different engine")]
    WrongEngine,
}

pub type Result<T> = std::result::Result<T, Error>;
