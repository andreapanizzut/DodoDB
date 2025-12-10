use thiserror::Error;

#[derive(Error, Debug)]
pub enum DodoError {
    #[error("Key not found")]
    NotFound,
}