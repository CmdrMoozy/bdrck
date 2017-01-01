use ::configuration;
use std::path;

extern crate tempfile;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct TestConfiguration {
    foo: String,
}

lazy_static! {
    static ref TEST_IDENTIFIER: configuration::Identifier = configuration::Identifier {
        application: "bdrck_config".to_owned(),
        name: "test".to_owned(),
    };
}

#[test]
fn test_persistence() {
    let file = tempfile::NamedTempFile::new().ok().unwrap();
    let path: path::PathBuf = file.path().to_owned();
    file.close().ok().unwrap();

    // Test that creating a configuration with an nonexistent file uses the default.
    let default = TestConfiguration { foo: "this is test data".to_owned() };
    configuration::new(TEST_IDENTIFIER.clone(),
                       default.clone(),
                       Some(path.to_str().unwrap()))
        .ok()
        .unwrap();
    assert_eq!(default, configuration::get(&TEST_IDENTIFIER).ok().unwrap());

    // Test that when we update the configuration, the new version is persisted,
    // and is re-loaded upon recreation.
    let updated = TestConfiguration { foo: "this is some other test data".to_owned() };
    configuration::set(&TEST_IDENTIFIER, updated.clone()).ok().unwrap();
    assert_eq!(updated, configuration::get(&TEST_IDENTIFIER).ok().unwrap());
    configuration::remove::<TestConfiguration>(&TEST_IDENTIFIER).ok().unwrap();
    configuration::new(TEST_IDENTIFIER.clone(),
                       default.clone(),
                       Some(path.to_str().unwrap()))
        .ok()
        .unwrap();
    assert_eq!(updated, configuration::get(&TEST_IDENTIFIER).ok().unwrap());

    // Test that we can then reset back to defaults.
    configuration::reset::<TestConfiguration>(&TEST_IDENTIFIER).ok().unwrap();
    assert_eq!(default, configuration::get(&TEST_IDENTIFIER).ok().unwrap());
}
