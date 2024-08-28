use std::collections::HashMap;

use libsecret::{Schema, SchemaAttributeType, SchemaFlags};

use crate::{Error, Identifier, KeychainItemBuilder, Result};

pub fn store(item: KeychainItemBuilder) -> Result<()> {
    let attributes = if let Some(username) = item.username {
        HashMap::from([("service", item.service), ("username", username)])
    } else {
        HashMap::from([("service", item.service)])
    };

    libsecret::password_store_sync(
        Some(&schema()),
        attributes,
        None,
        &format!(
            "Secret for '{}' on '{}'",
            item.username.unwrap_or(""),
            item.service
        ),
        item.secret,
        Option::<&gio::Cancellable>::None,
    )
    .map_err(Error)
}

pub fn load(identifier: Identifier) -> Result<Option<String>> {
    let attributes = if let Some(username) = identifier.username {
        HashMap::from([("service", identifier.service), ("username", username)])
    } else {
        HashMap::from([("service", identifier.service)])
    };

    Ok(libsecret::password_lookup_sync(
        Some(&schema()),
        attributes,
        Option::<&gio::Cancellable>::None,
    )
    .map_err(Error)?
    .map(From::from))
}

fn schema() -> Schema {
    const ATTRIBUTES: [(&str, SchemaAttributeType); 2] = [
        ("service", SchemaAttributeType::String),
        ("username", SchemaAttributeType::String),
    ];
    let attributes = HashMap::from(ATTRIBUTES);
    Schema::new(
        "org.freedesktop.Secret.Generic",
        SchemaFlags::NONE,
        attributes,
    )
}
