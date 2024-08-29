use core_foundation::data::CFData;
use security_framework::item::{
    add_item, delete_item, update_item, ItemAddOptions, ItemAddValue, ItemClass, ItemSearchOptions,
    ItemUpdateOptions, SearchResult,
};

use crate::{Class, Error, Identifier, KeychainItemBuilder, Result, UpdateOptions};

pub fn store(item: KeychainItemBuilder) -> Result<()> {
    let mut options = ItemAddOptions::new(ItemAddValue::Data {
        class: convert_class(item.class),
        data: CFData::from_buffer(item.secret.as_ref()),
    });
    if let Some(username) = item.username {
        options.set_account_name(username);
    }
    options.set_service(item.service);

    add_item(options.to_dictionary()).map_err(Error)
}

pub fn load(identifier: &Identifier) -> Result<Option<String>> {
    Ok(search_options(identifier)
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

pub fn update(identifier: &Identifier, options: UpdateOptions) -> Result<()> {
    let mut sf_options = ItemUpdateOptions::new();
    if let Some(service) = options.service {
        sf_options.set_service(service);
    }
    if let Some(secret) = options.secret {
        sf_options
            .set_value(ItemAddValue::Data {
                // This is ignored as we set class to `None`.
                class: ItemClass::generic_password(),
                data: CFData::from_buffer(secret.as_ref()),
            })
            .set_class(None);
    }
    if let Some(class) = options.class {
        sf_options.set_class(Some(convert_class(class)));
    }
    if let Some(username) = options.username {
        sf_options.set_account_name(username);
    }
    update_item(
        search_options(identifier).to_dictionary(),
        sf_options.to_dictionary(),
    )
    .map_err(Error)
}

pub fn delete(identifier: &Identifier) -> Result<()> {
    delete_item(search_options(identifier).to_dictionary()).map_err(Error)
}

fn search_options(identifier: &Identifier) -> ItemSearchOptions {
    let mut options = ItemSearchOptions::new();
    options
        .service(identifier.service)
        .class(convert_class(identifier.class));
    if let Some(username) = identifier.username {
        options.account(username);
    }
    options
}

fn convert_class(class: Class) -> ItemClass {
    match class {
        Class::Generic => ItemClass::generic_password(),
        Class::Internet => ItemClass::internet_password(),
    }
}
