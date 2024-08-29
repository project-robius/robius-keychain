use std::collections::HashMap;

use libsecret::{Schema, SchemaAttributeType, SchemaFlags};

use crate::{Error, Identifier, KeychainItemBuilder, Result, UpdateOptions};

pub fn store(item: KeychainItemBuilder) -> Result<()> {
    let attributes = attributes(item.service, item.username);

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

pub fn load(identifier: &Identifier) -> Result<Option<String>> {
    let attributes = attributes(identifier.service, identifier.username);
    Ok(libsecret::password_lookup_sync(
        Some(&schema()),
        attributes,
        Option::<&gio::Cancellable>::None,
    )
    .map_err(Error)?
    .map(From::from))
}

pub fn update(identifier: &Identifier, options: UpdateOptions) -> Result<()> {
    let secret = if let Some(secret) = options.secret {
        // TODO: Unnecessary allocation
        secret.to_owned()
    } else {
        // TODO: unwrap
        load(identifier)?.unwrap()
    };

    let mut builder =
        KeychainItemBuilder::new(options.service.unwrap_or(identifier.service), &secret);

    if let Some(username) = options.username {
        builder = builder.username(username);
    } else if let Some(username) = identifier.username {
        builder = builder.username(username);
    }

    store(builder)
}

pub fn delete(identifier: &Identifier) -> Result<()> {
    libsecret::password_clear_sync(
        Some(&schema()),
        attributes(identifier.service, identifier.username),
        Option::<&gio::Cancellable>::None,
    )
    .map_err(Error)
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

fn attributes<'a>(service: &'a str, username: Option<&'a str>) -> HashMap<&'a str, &'a str> {
    if let Some(username) = username {
        HashMap::from([("service", service), ("username", username)])
    } else {
        HashMap::from([("service", service)])
    }
}
