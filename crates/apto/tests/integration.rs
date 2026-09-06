use apto::{define_config, types::ArrayOf, types::Bool, types::Text, types::U32};

define_config! {
    @root {
        app_name: Text = "myapp".to_string(), "Application name",
        verbose: Bool = false, "Enable verbose output",
    }
    server {
        host: Text = "localhost".to_string(), "Server hostname",
        port: U32 = 8080, "Server port",
        tags: ArrayOf<Text> = Vec::new(), "Search tags",
    }
}

#[test]
fn root_fields_have_no_category() {
    let key = ConfigKey::app_name;
    assert!(key.category().is_none());
}

#[test]
fn category_fields_have_category() {
    let key = ConfigKey::host;
    assert_eq!(key.category(), Some(ConfigCategory::server));
}

#[test]
fn config_defaults() {
    let c = Config::default();
    assert_eq!(c.get_app_name(), "myapp");
    assert_eq!(c.get_verbose(), false);
    assert_eq!(c.get_host(), "localhost");
    assert_eq!(c.get_port(), 8080);
}

#[test]
fn config_get_value() {
    let c = Config::default();
    assert_eq!(c.get_value("app_name"), "myapp");
    assert_eq!(c.get_value("host"), "localhost");
    assert_eq!(c.get_value("verbose"), "false");
}

#[test]
fn config_set_value() {
    let mut c = Config::default();
    c.set_value("app_name", "newname").unwrap();
    assert_eq!(c.get_app_name(), "newname");
}

#[test]
fn config_reset_key() {
    let mut c = Config::default();
    c.set_value("app_name", "newname").unwrap();
    c.reset_key("app_name").unwrap();
    assert_eq!(c.get_app_name(), "myapp");
}

#[test]
fn config_description() {
    let c = Config::default();
    assert_eq!(c.get_description("app_name"), "Application name");
    assert_eq!(c.get_description("host"), "Server hostname");
    assert_eq!(c.get_description("nonexistent"), "unknown key");
}

#[test]
fn config_validate() {
    let c = Config::default();
    c.validate().unwrap();
}

#[test]
fn config_key_from_key() {
    assert!(ConfigKey::from_key("app_name").is_some());
    assert!(ConfigKey::from_key("host").is_some());
    assert!(ConfigKey::from_key("nonexistent").is_none());
}

#[test]
fn config_key_all_contains_all() {
    let all = ConfigKey::all();
    assert!(all.contains(&ConfigKey::app_name));
    assert!(all.contains(&ConfigKey::host));
    assert!(all.contains(&ConfigKey::port));
    assert!(all.contains(&ConfigKey::tags));
}

#[test]
fn config_categories() {
    let cats = ConfigCategory::all();
    assert_eq!(cats, &[ConfigCategory::server]);
}

#[test]
fn config_category_keys() {
    let keys = ConfigCategory::server.keys();
    assert!(keys.contains(&ConfigKey::host));
    assert!(keys.contains(&ConfigKey::port));
    assert!(keys.contains(&ConfigKey::tags));
}

#[test]
fn config_array_field_get() {
    let c = Config::default();
    let tags: Vec<String> = c.get_tags();
    assert!(tags.is_empty());
}

#[test]
fn config_array_field_set() {
    let mut c = Config::default();
    c.set_value("tags", "alpha, beta, gamma").unwrap();
    let tags = c.get_tags();
    assert_eq!(tags, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn config_toml_round_trip_defaults() {
    let config = Config::default();
    let toml = toml::to_string(&config).expect("serialization should succeed");
    let loaded: Config = toml::from_str(&toml).expect("deserialization should succeed");
    loaded.validate().unwrap();
    for key in ConfigKey::all() {
        assert_eq!(
            config.get_default_string(*key),
            loaded.get_default_string(*key),
            "default string mismatch for {:?}",
            key.as_str()
        );
    }
}

#[test]
fn config_toml_round_trip_modified_values() {
    let mut config = Config::default();
    config.set_value("app_name", "testapp").unwrap();
    config.set_value("port", "9090").unwrap();
    config.set_value("tags", "alpha, beta").unwrap();

    let toml = toml::to_string(&config).expect("serialization should succeed");
    let loaded: Config = toml::from_str(&toml).expect("deserialization should succeed");
    loaded.validate().unwrap();

    assert_eq!(loaded.get_app_name(), "testapp");
    assert_eq!(loaded.get_port(), 9090);
    assert_eq!(
        loaded.get_tags(),
        vec!["alpha".to_string(), "beta".to_string()]
    );
}

#[test]
fn config_toml_empty_deserializes_to_defaults() {
    let loaded: Config = toml::from_str("").expect("empty TOML should deserialize");
    loaded.validate().unwrap();
    let default = Config::default();
    for key in ConfigKey::all() {
        assert_eq!(
            loaded.get_value(key.as_str()),
            default.get_value(key.as_str()),
            "value mismatch for {}",
            key.as_str()
        );
    }
}

#[test]
fn config_save_to_temp_file_and_reload() {
    let tmp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let config_path = tmp_dir.path().join("config.toml");

    let mut config = Config::default();
    config.set_value("host", "example.com").unwrap();
    config.set_value("port", "3000").unwrap();
    config.set_value("verbose", "true").unwrap();

    config.validate().unwrap();
    let toml = toml::to_string(&config).expect("serialization failed");
    std::fs::write(&config_path, &toml).expect("write failed");

    let content = std::fs::read_to_string(&config_path).expect("read failed");
    let loaded: Config = toml::from_str(&content).expect("deserialization failed");
    loaded.validate().unwrap();

    assert_eq!(loaded.get_host(), "example.com");
    assert_eq!(loaded.get_port(), 3000);
    assert_eq!(loaded.get_verbose(), true);

    std::fs::remove_file(&config_path).ok();
}

#[test]
fn config_set_reset_round_trip() {
    let mut config = Config::default();
    config.set_value("port", "4000").unwrap();
    assert_eq!(config.get_port(), 4000);

    let toml = toml::to_string(&config).expect("serialization");
    let loaded: Config = toml::from_str(&toml).expect("deserialization");
    assert_eq!(loaded.get_port(), 4000);

    // Reset and verify it goes back to default
    let mut reset = loaded;
    reset.reset_key("port").unwrap();
    assert_eq!(reset.get_port(), 8080);
}
