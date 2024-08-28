fn main() {
    let secret = "hunter2";
    let id = robius_keychain::KeychainItemBuilder::new("hello_world", &secret)
        .username("username")
        .store()
        .expect("failed to create keychain item");

    assert_eq!(
        secret,
        robius_keychain::load(id)
            .expect("failed to search keychain")
            .expect("found no keychain items")
    );
    assert_eq!(
        secret,
        &robius_keychain::load(robius_keychain::Identifier {
            service: "hello_world",
            username: Some("username"),
            class: robius_keychain::Class::Generic,
        })
        .expect("failed to search keychain")
        .expect("found no keychain items")
    );
}
