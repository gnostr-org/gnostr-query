use serde_json::{Result, Value};
//use serde::de::Error;
use serde::ser::Error;

fn extract_elements(json_str: &str, keys: &[&str]) -> Result<Value> {
    let json: Value = serde_json::from_str(json_str)?;

    match json {
        Value::Object(map) => {
            let mut extracted = serde_json::Map::new();
            for key in keys {
                if let Some(value) = map.get(*key) {
                    extracted.insert(key.to_string(), value.clone());
                }
            }
            Ok(Value::Object(extracted))
        }
        _ => Err(serde_json::Error::custom("Input is not a JSON object")),
    }
}

fn main() {
    let json_str = r#"
        {
            "name": "John Doe",
            "age": 30,
            "city": "New York",
            "is_active": true,
            "address": {
                "street": "123 Main St",
                "zip": "10001"
            }
        }
    "#;

    let keys_to_extract = ["name", "age", "address"];

    match extract_elements(json_str, &keys_to_extract) {
        Ok(extracted_json) => {
            println!("Extracted JSON: {}", extracted_json);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }

    let json_str_array = r#"[
        {"name": "John Doe", "age": 30},
        {"name": "Jane Smith", "age": 25}
    ]"#;

    let keys_array = ["name"];

    let result_array: Result<Vec<Value>> =
        serde_json::from_str(json_str_array).map(|array: Vec<Value>| {
            array
                .into_iter()
                .map(|value| match value {
                    Value::Object(map) => {
                        let mut extracted = serde_json::Map::new();
                        for key in keys_array {
                            if let Some(val) = map.get(key) {
                                extracted.insert(key.to_string(), val.clone());
                            }
                        }
                        Value::Object(extracted)
                    }
                    _ => Value::Null,
                })
                .collect()
        });

    match result_array {
        Ok(extracted_array) => println!(
            "Extracted Array: {}",
            serde_json::to_string_pretty(&extracted_array).unwrap()
        ),
        Err(e) => eprintln!("Error extracting from array: {}", e),
    }
}
