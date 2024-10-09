use std::{io, path::PathBuf};

use robius_directories::ProjectDirs;

use crate::{error::ErrorVariant, Error, Identifier, KeychainItemBuilder, Result, UpdateOptions};

const SEPARATOR: char = '\x1f';

pub fn store(item: KeychainItemBuilder) -> Result<()> {
    std::fs::create_dir_all(dir()?)?;

    let identifier = Identifier {
        service: item.service,
        class: item.class,
        username: item.username,
    };

    std::fs::write(file(&identifier)?, item.secret)?;
    Ok(())
}

pub fn load(identifier: &Identifier) -> Result<Option<String>> {
    let bytes = match std::fs::read(file(identifier)?) {
        Ok(bytes) => bytes,
        Err(error) => {
            return match error.kind() {
                io::ErrorKind::NotFound => Ok(None),
                _ => Err(Error::from(error)),
            }
        }
    };

    match String::from_utf8(bytes) {
        Ok(string) => Ok(Some(string)),
        Err(error) => Err(Error::from(error)),
    }
}

pub fn update(identifier: &Identifier, options: UpdateOptions) -> Result<()> {
    let new_identifier = Identifier {
        service: options.service.unwrap_or(identifier.service),
        class: options.class.unwrap_or(identifier.class),
        username: match options.username {
            Some(username) => Some(username),
            None => identifier.username,
        },
    };

    let old_path = file(identifier)?;
    let new_path = file(&new_identifier)?;

    if old_path != new_path {
        std::fs::rename(old_path, &new_path)?;
    }

    if let Some(secret) = options.secret {
        std::fs::write(new_path, secret)?;
    }

    Ok(())
}

pub fn delete(identifier: &Identifier) -> Result<()> {
    std::fs::remove_file(file(identifier)?).map_err(Error::from)
}

fn file(identifier: &Identifier) -> Result<PathBuf> {
    dir().map(|dir| dir.join(file_name(identifier)))
}

fn file_name(identifier: &Identifier) -> String {
    if let Some(username) = identifier.username {
        // TODO: Do we bother documenting this. It's kind of an internal implementation
        // detail. Note that the username and service can contain \x1f.
        //
        // TODO: Would we rather use a printable character?
        format!("{}{SEPARATOR}{}", username, identifier.service)
    } else {
        identifier.service.to_owned()
    }
}

fn dir() -> Result<PathBuf> {
    Ok(ProjectDirs::from("", "", "")
        .ok_or(Error(ErrorVariant::Directories))?
        .data_dir()
        .join("robius-keychain"))
}
