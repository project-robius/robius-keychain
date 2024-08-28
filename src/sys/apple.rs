use core_foundation::data::CFData;
use security_framework::item::{
    add_item, ItemAddOptions, ItemAddValue, ItemClass, ItemSearchOptions, SearchResult,
};

use crate::{Class, Error, Identifier, KeychainItemBuilder, Result};

pub fn store(item: KeychainItemBuilder) -> Result<()> {
    let mut options = ItemAddOptions::new(ItemAddValue::Data {
        class: match item.class {
            Class::Generic => ItemClass::generic_password(),
            Class::Internet => ItemClass::internet_password(),
        },
        data: CFData::from_buffer(item.secret.as_ref()),
    });
    if let Some(username) = item.username {
        options.set_account_name(username);
    }
    options.set_service(item.service);

    add_item(options.to_dictionary()).map_err(Error)
}

pub fn load(identifier: Identifier) -> Result<Option<String>> {
    let mut options = ItemSearchOptions::new();
    options.service(identifier.service);
    if let Some(username) = identifier.username {
        options.account(username);
    }

    Ok(options
        .class(ItemClass::generic_password())
        .load_data(true)
        .search()
        .map_err(Error)?
        .into_iter()
        .next()
        .and_then(|r| match r {
            SearchResult::Data(d) => String::from_utf8(d).ok(),
            // TODO: Log error?
            _ => None,
        }))
}
