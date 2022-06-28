
fn main() -> Result<(), pepper::Error> {
    let input = r#"{"key1": "value","key2": 123.0,"key3": {"key1": "value", "key2": 123.0}}"#;
    match pepper::Parser::new().parse(&input) {
        Ok(Some(val)) => println!("{:?}", val),
        Ok(None) => println!("end of file"),
        Err(e) => eprintln!("{}",e),
    }

    Ok(())
}
