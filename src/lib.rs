use jsonschema::{Draft, JSONSchema};
use serde_json::json;
use serde_json::Value;

#[derive(Debug)]
pub struct CJValidator {
    j: Value,
    is_cityjson: Option<bool>,
    version: Option<i32>,
    is_schema_valid: Option<bool>,
}

impl CJValidator {
    pub fn from_str(s: &str) -> Self {
        let j: Value = serde_json::from_str(&s).unwrap();
        let mut v = CJValidator {
            j: json!(null),
            is_cityjson: Some(true),
            version: None,
            is_schema_valid: None,
        };
        if j.is_null() {
            v.is_cityjson = Some(false);
            return v;
        }
        if j["type"] != "CityJSON" {
            v.is_cityjson = Some(false);
        }
        //-- which cityjson version
        if j["version"] == "1.1" {
            v.version = Some(11);
        } else if j["version"] == 1.0 {
            v.version = Some(10);
        }
        v.j = j;
        v
    }

    pub fn validate_schema(&self) -> Result<(), Vec<String>> {
        if self.j.is_null() {
            return Err(vec!["Not a valid JSON file".to_string()]);
        }
        if self.is_cityjson.is_none() {
            println!("here");
            return Err(vec!["Not a CityJSON file".to_string()]);
        }
        if self.version.is_none() {
            println!("here2");
            return Err(vec!["Not a supported CityJSON version".to_string()]);
        }
        if (self.version.unwrap() < 10) || (self.version.unwrap() > 11) {
            return Err(vec!["CityJSON version not supported".to_string()]);
        }
        //-- fetch the correct schema
        let mut schema_str = include_str!("../schemas/10/cityjson.min.schema.json");
        if self.version.unwrap() == 11 {
            schema_str = include_str!("../schemas/11/cityjson.min.schema.json");
        }
        let schema = serde_json::from_str(schema_str).unwrap();

        let compiled = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema)
            .expect("A valid schema");
        let result = compiled.validate(&self.j);
        let mut ls_errors: Vec<String> = Vec::new();
        if let Err(errors) = result {
            for error in errors {
                let s: String = format!("{} [path:{}]", error, error.instance_path);
                ls_errors.push(s);
            }
            return Err(ls_errors);
        } else {
            return Ok(());
        }
    }
}
