use std::{error, fmt};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
#[cfg(target_vendor = "apple")]
pub struct Error(pub(crate) security_framework::base::Error);

#[derive(Debug, Clone)]
#[cfg(target_os = "linux")]
pub struct Error(pub(crate) glib::error::Error);

#[derive(Debug, Clone)]
#[cfg(target_os = "windows")]
pub struct Error(pub(crate) windows_result::Error);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.0)
    }
}
