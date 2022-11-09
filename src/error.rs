use std;
use std::fmt::Debug;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Error {
    pub message: String,
    pub source_error: Option<Box<dyn std::error::Error>>
}

impl Error {
    pub fn new<S: ToString>(message: S) -> Self {
        Error {
            message: message.to_string(),
            source_error: None,
        }
    }
}

pub trait ConvertError {
    fn into_error<S: ToString>(self, message: S) -> Error;
}

impl<T: 'static + std::error::Error> ConvertError for T {
    fn into_error<S: ToString>(self, message: S) -> Error {
        Error {
            message: message.to_string(),
            source_error: Some(Box::new(self)),
        }
    }
}

pub trait ConvertResult<T> {
    fn into_error<S: ToString>(self, message: S) -> Result<T, Error>;
}

impl<T, E: ConvertError> ConvertResult<T> for Result<T, E> {
    fn into_error<S: ToString>(self, message: S) -> Result<T, Error> {
        self.map_err(|e| e.into_error::<S>(message))
    }
}

// impl<T> ConvertResult<T> for Option<T> {
//     fn into_error<S: ToString>(self, message: S) -> Result<T, Error> {
//         self.ok_or(Error {
//             message: message.to_string(),
//             source_error: None
//         })
//     }
// }
