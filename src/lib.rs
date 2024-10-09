//! A library for managing keychain items.
//!
//! # Examples
//!
//! ```
//! let secret = "hunter2";
//! 
//! let id = robius_keychain::KeychainItemBuilder::new("hello_world", &secret)
//!     .username("username")
//!     .store()
//!     .expect("failed to create keychain item");
//! 
//! assert_eq!(
//!     secret,
//!     id.load()
//!         .expect("failed to search keychain")
//!         .expect("found no keychain items")
//! );
//! // Alternatively we can recreate the Identifier struct.
//! assert_eq!(
//!     secret,
//!     robius_keychain::Identifier {
//!         service: "hello_world",
//!         username: Some("username"),
//!         class: robius_keychain::Class::Generic,
//!     }
//!     .load()
//!     .expect("failed to search keychain")
//!     .expect("found no keychain items")
//! );
//! 
//! // If we don't update the service or username, then the old id will still be
//! // valid.
//! let id = id
//!     .update(
//!         robius_keychain::UpdateOptions::new()
//!             .username("new username")
//!             .secret("hunter 3"),
//!     )
//!     .expect("failed to update keychain item");
//! 
//! id.delete().expect("failed to delete keychain item");
//! ```

mod error;
mod sys;

pub use error::{Error, Result};

/// Builder to add an item to the keychain.
///
/// See the crate-level docs for more information.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct KeychainItemBuilder<'a> {
    service: &'a str,
    secret: &'a str,
    class: Class,
    username: Option<&'a str>,
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
    /// Creates a new `KeychainItemBuilder.
    pub fn new(service: &'a str, secret: &'a str) -> Self {
        Self {
            service,
            secret,
            class: Class::default(),
            username: None,
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

#[derive(Clone, Debug, Default)]
pub struct UpdateOptions<'a> {
    service: Option<&'a str>,
    secret: Option<&'a str>,
    class: Option<Class>,
    username: Option<&'a str>,
}

impl<'a> UpdateOptions<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the service for the keychain item.
    pub fn service(mut self, service: &'a str) -> Self {
        self.service = Some(service);
        self
    }

    /// Sets the secret for the keychain item.
    pub fn secret(mut self, secret: &'a str) -> Self {
        self.secret = Some(secret);
        self
    }

    /// Sets the class for the keychain item.
    pub fn class(mut self, class: Class) -> Self {
        self.class = Some(class);
        self
    }

    /// Sets the username for the keychain item.
    pub fn username(mut self, username: &'a str) -> Self {
        self.username = Some(username);
        self
    }
}

// An identifier returned by [`KeychainItemBuilder::store`] to later [`load`]
// the item.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Identifier<'a> {
    pub service: &'a str,
    pub username: Option<&'a str>,
    pub class: Class,
}

impl<'a> Identifier<'a> {
    // Poor man's `ToOwned` impl. We can't impl ToOwned because OwnedIdentifier
    // can't implement Borrow.
    pub fn to_owned(&self) -> OwnedIdentifier {
        OwnedIdentifier {
            service: self.service.to_owned(),
            username: self.username.map(ToOwned::to_owned),
            class: self.class,
        }
    }

    /// See the crate-level docs for more information.
    pub fn load(&self) -> Result<Option<String>> {
        sys::load(self)
    }

    pub fn update(&self, options: UpdateOptions<'a>) -> Result<Self> {
        let new_id = Self {
            service: options.service.unwrap_or(self.service),
            username: options.username.or(self.username),
            class: options.class.unwrap_or(self.class),
        };
        sys::update(self, options)?;
        Ok(new_id)
    }

    pub fn delete(&self) -> Result<()> {
        sys::delete(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnedIdentifier {
    pub service: String,
    pub username: Option<String>,
    pub class: Class,
}

impl OwnedIdentifier {
    fn as_ref(&self) -> Identifier {
        Identifier {
            service: self.service.as_ref(),
            username: self.username.as_ref().map(|u| u.as_ref()),
            class: self.class,
        }
    }

    /// See the crate-level docs for more information.
    pub fn load(&self) -> Result<Option<String>> {
        self.as_ref().load()
    }

    pub fn update<'a>(&'a self, options: UpdateOptions<'a>) -> Result<Identifier<'a>> {
        self.as_ref().update(options)
    }

    pub fn delete(&self) -> Result<()> {
        self.as_ref().delete()
    }
}
