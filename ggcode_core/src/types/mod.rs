use std::error::Error;

pub type ErrorBox = Box<dyn Error>;
pub type AppResult<T> = Result<T, ErrorBox>;
