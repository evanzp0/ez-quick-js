use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Property error: {0}")]
    PropertyError(String),
    #[error("Bad type error: {0}")]
    BadType(String),
    #[error("Value error: {0}")]
    ValueError(String),
}

impl Error {
    pub fn bad_type<T1, T2>(msg: &str) -> Self {
        let t1 = std::any::type_name::<T1>().to_owned();
        let t2 = std::any::type_name::<T2>().to_owned();
        let msg = format!("{msg}({t1}, {t2})");
        Error::BadType(msg)
    }
}