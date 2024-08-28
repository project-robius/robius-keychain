use std::{ptr, slice, str};

use windows::{
    core::PWSTR,
    Win32::{
        Foundation::FILETIME,
        Security::Credentials::{
            CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_FLAGS, CRED_PERSIST, CRED_TYPE,
        },
    },
};

use crate::{Class, Error, Identifier, KeychainItemBuilder, Result};

pub fn store(item: KeychainItemBuilder) -> Result<()> {
    // let mut attributes = Vec::with_capacity(item.attributes.len());
    // let mut keys = Vec::with_capacity(item.attributes.len());

    // XXX: `item.attributes` must not be dropped until after we have called
    // `CredWriteW`. This is a load-bearing ampersand.
    // for (key, value) in &item.attributes {
    //     let (v, key) = w(key);
    //     // XXX: We must only drop the `PWSTR` backing vector after calling
    // `CredWriteW`.     // Hence we store it in a vector till the end of the
    // function.     keys.push(v);

    //     attributes.push(CREDENTIAL_ATTRIBUTEW {
    //         Keyword: key,
    //         Flags: 0,
    //         ValueSize: value.len() as u32,
    //         Value: value.as_ptr() as *mut _,
    //     });
    // }

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

pub fn load(identifier: Identifier) -> Result<Option<String>> {
    let (_target, target) = target(identifier.service, identifier.username);
    let ty = ty(identifier.class);
    let mut ptr = ptr::null_mut();

    unsafe { CredReadW(target, ty, 0, &mut ptr as *mut _) }.map_err(Error)?;

    let cred: &CREDENTIALW = unsafe { &*ptr };

    // Copy out the secret so that we can free the struct.
    let sl =
        unsafe { slice::from_raw_parts(cred.CredentialBlob, cred.CredentialBlobSize as usize) };
    let s = str::from_utf8(sl).ok().map(|s| s.to_owned());

    unsafe { CredFree(ptr as *const _) };

    Ok(s)
}

fn target(service: &str, username: Option<&str>) -> (Vec<u16>, PWSTR) {
    if let Some(username) = username {
        // TODO: Do we bother documenting this. It's kind of an internal implementation
        // detail.
        w(&format!("{}@{}", username, service))
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
