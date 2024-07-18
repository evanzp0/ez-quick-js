use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Bad type error: {0}")]
    BadType(String)
}

impl Error {
    pub fn bad_type<T1, T2>(msg: &str) -> Self {
        let t1 = std::any::type_name::<T1>().to_owned();
        let t2 = std::any::type_name::<T2>().to_owned();
        let msg = format!("{msg}({t1}, {t2})");
        Error::BadType(msg)
    }
}