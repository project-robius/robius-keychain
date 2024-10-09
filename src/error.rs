use std::{error, fmt};

pub type Result<T, E = Error> = std::result::Result<T, E>;

cfg_if::cfg_if! {
    if #[cfg(target_os = "android")] {
        #[derive(Debug)]
        pub struct Error(pub(crate) ErrorVariant);

        #[derive(Debug)]
        pub(crate) enum ErrorVariant {
            Io(std::io::Error),
            Utf8(std::string::FromUtf8Error),
            Directories,
        }

        impl From<std::io::Error> for Error {
            fn from(error: std::io::Error) -> Self {
                Self(ErrorVariant::Io(error))
            }
        }

        impl From<std::string::FromUtf8Error> for Error {
            fn from(error: std::string::FromUtf8Error) -> Self {
                Self(ErrorVariant::Utf8(error))
            }
        }
    } else if #[cfg(target_vendor = "apple")] {
        #[derive(Debug)]
        pub struct Error(pub(crate) security_framework::base::Error);
    } else if #[cfg(target_os = "linux")] {
        #[derive(Debug)]
        pub struct Error(pub(crate) glib::error::Error);
    } else if #[cfg(target_os = "windows")] {
        #[derive(Debug)]
        pub struct Error(pub(crate) windows_result::Error);
    } else {
        #[derive(Debug)]
        pub struct Error(());
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "android")] {
                match self.0 {
                    ErrorVariant::Io(ref e) => write!(f, "{e}"),
                    ErrorVariant::Utf8(ref e) => write!(f, "{e}"),
                    ErrorVariant::Directories => write!(f, "todo"),
                }
            } else if #[cfg(target_vendor = "apple")] {
                write!(f, "{}", self.0)
            } else if #[cfg(target_os = "linux")] {
                write!(f, "{}", self.0)
            } else if #[cfg(target_os = "windows")] {
                write!(f, "{}", self.0)
            } else {
                write!(f, "error")
            }
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "android")] {
                match self.0 {
                    ErrorVariant::Io(ref e) => Some(e),
                    ErrorVariant::Utf8(ref e) => Some(e),
                    ErrorVariant::Directories => None,
                }
            } else if #[cfg(target_vendor = "apple")] {
                Some(&self.0)
            } else if #[cfg(target_os = "linux")] {
                Some(&self.0)
            } else if #[cfg(target_os = "windows")] {
                Some(&self.0)
            } else {
                None
            }
        }
    }
}
