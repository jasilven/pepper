use std::collections::HashMap;
use pepper::{Value,Parser};

#[test]
fn basic_parsing() {
    let mut hm = HashMap::new();
    hm.insert("key1".to_string(), Value::String("value".to_string()));
    hm.insert("key2".to_string(), Value::Number(123.0));
    hm.insert("key3".to_string(), Value::Object(hm.clone()));

    let input = r#"{"key1": "value", "key2": 123.0, "key3": {"key1": "value", "key2": 123.0}}"#;
    let value = Parser::new().parse(&input).unwrap().unwrap();
    assert_eq!(value, Value::Object(hm));
}
