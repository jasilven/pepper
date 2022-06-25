use std::collections::HashMap;

#[test]
fn basic_parsing() {
    let mut hm = HashMap::new();
    hm.insert("key1".to_string(), pepper::Value::String("value".to_string()));
    hm.insert("key2".to_string(), pepper::Value::Number(123.0));
    hm.insert("key3".to_string(), pepper::Value::Object(hm.clone()));

    let input = r#"{"key1": "value", "key2": 123.0, "key3": {"key1": "value", "key2": 123.0}}"#;
    let value = pepper::Parser::new().parse(&input).unwrap().unwrap();
    assert_eq!(value, pepper::Value::Object(hm));
}
