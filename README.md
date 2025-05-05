# robius-keychain
A library for managing keychain items.

## Dependencies
### Linux
Require libsecret development libraries
```bash
# Ubuntu
sudo apt install libsecret-1-dev
# Fedora
sudo dnf install libsecret-devel
```

## Examples
```rust
let secret = "hunter2";

let id = robius_keychain::KeychainItemBuilder::new("hello_world", &secret)
    .username("username")
    .store()
    .expect("failed to create keychain item");

assert_eq!(
    secret,
    id.load()
        .expect("failed to search keychain")
        .expect("found no keychain items")
);
// Alternatively we can recreate the Identifier struct.
assert_eq!(
    secret,
    robius_keychain::Identifier {
        service: "hello_world",
        username: Some("username"),
        class: robius_keychain::Class::Generic,
    }
    .load()
    .expect("failed to search keychain")
    .expect("found no keychain items")
);

// If we don't update the service or username, then the old id will still be
// valid.
let id = id
    .update(
        robius_keychain::UpdateOptions::new()
            .username("new username")
            .secret("hunter 3"),
    )
    .expect("failed to update keychain item");

id.delete().expect("failed to delete keychain item");
```
