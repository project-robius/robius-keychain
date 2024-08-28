//! A library for managing keychain items.
//!
//! # Examples
//!
//! ```
//! let secret = "hunter2";
//! let id = robius_keychain::KeychainItemBuilder::new("hello_world", &secret)
//!     .class(robius_keychain::Class::Generic)
//!     .username("username")
//!     .store()
//!     .expect("failed to create keychain item");
//!
//! assert_eq!(
//!     secret,
//!     robius_keychain::load(id)
//!         .expect("failed to search keychain")
//!         .expect("found no keychain items")
//! );
//! assert_eq!(
//!     secret,
//!     &robius_keychain::load(robius_keychain::Identifier {
//!         service: "hello_world",
//!         username: Some("username"),
//!         class: robius_keychain::Class::Generic,
//!     })
//!     .expect("failed to search keychain")
//!     .expect("found no keychain items")
//! );
//! ```

mod error;
mod sys;

pub use error::{Error, Result};

/// Builder to add an item to the keychain.
///
/// See the crate-level docs for more information.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeychainItemBuilder<'a> {
    service: &'a str,
    secret: &'a str,
    class: Class,
    username: Option<&'a str>,
    synchronizable: Option<bool>,
    // comment: Option<&'a str>,
    // attributes: Vec<(&'a str, &'a [u8])>,
}

/// The class of the keychain item.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Class {
    Generic,
    Internet,
    // Certificate,
}

impl Default for Class {
    fn default() -> Self {
        Self::Generic
    }
}

impl<'a> KeychainItemBuilder<'a> {
    pub fn new(service: &'a str, secret: &'a str) -> Self {
        Self {
            service,
            secret,
            class: Class::default(),
            username: None,
            synchronizable: None,
        }
    }

    /// Sets the class for the keychain item.
    pub fn class(mut self, class: Class) -> Self {
        self.class = class;
        self
    }

    /// Sets the username for the keychain item.
    pub fn username(mut self, username: &'a str) -> Self {
        self.username = Some(username);
        self
    }

    pub fn synchronizable(mut self, synchronizable: bool) -> Self {
        self.synchronizable = Some(synchronizable);
        self
    }

    // pub fn comment(&mut self, comment: &'a str) {
    //     self.comment = comment;
    // }

    // pub fn add_attribute<T>(&mut self, key: &'a str, value: &'a T)
    // where
    //     T: AsRef<[u8]>,
    // {
    //     self.attributes.push((key, value.as_ref()));
    // }

    /// Stores the item in the keychain.
    pub fn store(self) -> Result<Identifier<'a>> {
        let identifier = Identifier {
            service: self.service,
            username: self.username,
            class: self.class,
        };
        sys::store(self)?;
        Ok(identifier)
    }
}

// An identifier returned by [`KeychainItemBuilder::store`] to later [`load`]
// the item.
pub struct Identifier<'a> {
    pub service: &'a str,
    pub username: Option<&'a str>,
    pub class: Class,
}

// TODO: Add owned version of identifier for less hassle. It doesn't really work
// with traits though because we can't return a reference to an `Identifier`.

// impl<'a> borrow::ToOwned for Identifier<'a> {
//     type Owned = OwnedIdentifier;
//
//     fn to_owned(&self) -> Self::Owned {
//         OwnedIdentifier {
//             service: self.to_owned(),
//             username: self.username.map(|username| username.to_owned()),
//         }
//     }
// }
//
// pub struct OwnedIdentifier {
//     pub service: String,
//     pub username: String,
// }
//
// impl<'a> borrow::Borrow<Identifier<'a>> for OwnedIdentifier {
//     fn borrow(&self) -> &Identifier {
//         &Identifier {
//             service: self.service.borrow(),
//             username: self.username.borrow(),
//         }
//     }
// }

/// Loads an item from the keychain given an [`Identifier`].
///
/// See the crate-level docs for more information.
pub fn load(identifier: Identifier) -> Result<Option<String>> {
    sys::load(identifier)
}
