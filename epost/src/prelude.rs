// ! Crate prelude

pub use crate::error::Error;

pub type Result<T> = core::result::Result<T, Error>;

// Generic Wrapper tuple struct for new type.
pub struct Wrapper<T>(pub T);
