use std::{ptr, slice, str};

use windows::{
    core::PWSTR,
    Win32::{
        Foundation::FILETIME,
        Security::Credentials::{
            CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_FLAGS, CRED_PERSIST,
            CRED_TYPE,
        },
    },
};

use crate::{Class, Error, Identifier, KeychainItemBuilder, Result, UpdateOptions};

const TARGET_NAME_SEPARATOR: char = '\x1f';

pub fn store(item: KeychainItemBuilder) -> Result<()> {
    let (_target, target) = target(item.service, item.username);
    let (_user_name, user_name) = if let Some(username) = item.username {
        w(username)
    } else {
        (Vec::new(), PWSTR::null())
    };

    let credentials = CREDENTIALW {
        Flags: CRED_FLAGS(0x0),
        Type: ty(item.class),
        TargetName: target,
        Comment: PWSTR::null(),
        LastWritten: FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        },
        CredentialBlobSize: item.secret.len() as u32,
        CredentialBlob: item.secret.as_ptr() as *mut _,
        // TODO: Option to only persist for session. Could be an interesting feature.
        Persist: CRED_PERSIST(0x2),
        // AttributeCount: attributes.len() as u32,
        // Attributes: attributes.as_mut_ptr(),
        AttributeCount: 0,
        Attributes: ptr::null_mut(),
        TargetAlias: PWSTR::null(),
        UserName: user_name,
    };

    unsafe { CredWriteW(&credentials as *const _, 0) }.map_err(Error)
}

pub fn load(identifier: &Identifier) -> Result<Option<String>> {
    // TODO: Don't propagate notfound error or something.
    let ptr = load_(identifier)?;
    let cred = unsafe { &*ptr };

    // Copy out the secret so that we can free the struct.
    let sl =
        unsafe { slice::from_raw_parts(cred.CredentialBlob, cred.CredentialBlobSize as usize) };
    let s = str::from_utf8(sl).ok().map(|s| s.to_owned());

    unsafe { CredFree(ptr as *const _) };

    Ok(s)
}

pub fn update(identifier: &Identifier, options: UpdateOptions) -> Result<()> {
    let previous_ptr = load_(identifier)?;
    let previous_cred = unsafe { &*previous_ptr };

    // TODO: Explain
    let f = |service, username| {
        delete(identifier)?;
        Ok(target(service, username))
    };

    // TODO: Unwraps
    let (_target, target) = match (options.service, options.username) {
        (Some(service), username) => f(service, username)?,
        (None, Some(username)) => {
            // TODO: Could also just use the service stored in the identifier.

            // SAFETY: Pointer supplied by Windows must be valid.
            let previous_target_name = unsafe { previous_cred.TargetName.to_string() }.unwrap();
            // SAFETY: Pointer supplied by Windows must be valid.
            let previous_username = unsafe { previous_cred.UserName.to_string() }.unwrap();
            let service = previous_target_name
                .strip_prefix(&previous_username)
                .unwrap()
                .strip_prefix(TARGET_NAME_SEPARATOR)
                .unwrap();
            f(service, Some(username))?
        }
        (None, None) => (Vec::new(), previous_cred.TargetName),
    };

    let (secret_len, secret_ptr) = if let Some(secret) = options.secret {
        (secret.len() as u32, secret.as_ptr() as *mut _)
    } else {
        (
            previous_cred.CredentialBlobSize,
            previous_cred.CredentialBlob,
        )
    };

    let (_user_name, user_name) = if let Some(username) = options.username {
        w(username)
    } else {
        (Vec::new(), previous_cred.UserName)
    };

    let credentials = CREDENTIALW {
        Flags: CRED_FLAGS(0x0),
        Type: options.class.map(ty).unwrap_or(previous_cred.Type),
        TargetName: target,
        Comment: PWSTR::null(),
        LastWritten: FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        },
        CredentialBlobSize: secret_len,
        CredentialBlob: secret_ptr,
        Persist: CRED_PERSIST(0x2),
        AttributeCount: 0,
        Attributes: ptr::null_mut(),
        TargetAlias: PWSTR::null(),
        UserName: user_name,
    };

    unsafe { CredWriteW(&credentials as *const _, 0) }.map_err(Error)?;
    unsafe { CredFree(previous_ptr as *const _) };

    Ok(())
}

pub fn delete(identifier: &Identifier) -> Result<()> {
    let (_target, target) = target(identifier.service, identifier.username);
    unsafe { CredDeleteW(target, ty(identifier.class), 0) }.map_err(Error)
}

fn load_(identifier: &Identifier) -> Result<*const CREDENTIALW> {
    let (_target, target) = target(identifier.service, identifier.username);
    let ty = ty(identifier.class);
    let mut ptr = ptr::null_mut();

    // TODO: Return option if this is a notfound error.
    unsafe { CredReadW(target, ty, 0, &mut ptr as *mut _) }.map_err(Error)?;

    Ok(ptr)
}

fn target(service: &str, username: Option<&str>) -> (Vec<u16>, PWSTR) {
    if let Some(username) = username {
        // TODO: Do we bother documenting this. It's kind of an internal implementation
        // detail. Note that the username and service can contain \x1f, because when
        // updating we strip the username rather than splitting on \x1f.
        //
        // TODO: Would we rather use a printable character?
        w(&format!("{}{TARGET_NAME_SEPARATOR}{}", username, service))
    } else {
        w(service)
    }
}

fn ty(class: Class) -> CRED_TYPE {
    CRED_TYPE(match class {
        // CRED_TYPE_GENERIC_CERTIFICATE
        // Class::Certificate => 0x5,
        // CRED_TYPE_GENERIC
        _ => 0x1,
    })
}

// The returned vector must not be dropped until the `PWSTR` is no longer in
// use.
fn w(s: &str) -> (Vec<u16>, PWSTR) {
    let mut utf16 = s.encode_utf16().collect::<Vec<_>>();
    utf16.push(0);
    let pwstr = PWSTR(utf16.as_mut_ptr());
    (utf16, pwstr)
}
