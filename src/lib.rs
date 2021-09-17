use jsonschema::{Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug)]
struct VertexF {
    x: f64,
    y: f64,
    z: f64,
}
#[derive(Serialize, Deserialize, Debug)]
struct VertexI {
    x: u32,
    y: u32,
    z: u32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Doc {
    #[serde(with = "::serde_with::rust::maps_duplicate_key_is_error")]
    CityObjects: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct CJValidator {
    j: Value,
    duplicate_keys: bool,
}

impl CJValidator {
    pub fn from_str(s: &str) -> Result<Self, String> {
        let mut v = CJValidator {
            j: json!(null),
            duplicate_keys: false,
        };
        let re: Result<Value, _> = serde_json::from_str(&s);
        if re.is_err() {
            // println!("errors: {:?}", re.as_ref().err().unwrap());
            return Err(re.err().unwrap().to_string());
        }
        let j: Value = re.unwrap();
        v.j = j;
        //-- check for duplicate keys in CO object
        let re: Result<Doc, _> = serde_json::from_str(&s);
        if re.is_err() {
            v.duplicate_keys = true;
        }
        Ok(v)
    }

    pub fn validate_schema(&self) -> Vec<String> {
        if self.j.is_null() {
            return vec!["Not a valid JSON file".to_string()];
        }
        //-- which cityjson version
        let version;
        if self.j["version"] == "1.1" {
            version = 11;
        } else if self.j["version"] == 1.0 {
            version = 10;
        } else {
            return vec!["CityJSON version not supported".to_string()];
        }
        //-- fetch the correct schema
        let mut schema_str = include_str!("../schemas/10/cityjson.min.schema.json");
        if version == 11 {
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
        }
        //-- duplicate keys
        if ls_errors.is_empty() && self.duplicate_keys {
            ls_errors.push("Duplicate keys in 'CityObjects'".to_string())
        }
        return ls_errors;
    }

    // parent_children_consistency
    pub fn parent_children_consistency(&self) -> Vec<String> {
        // if (self.is_schema_valid.is_none()) || (self.is_schema_valid.unwrap() != true) {
        //     return vec!["This is not schema valid (or hasn't been tested yet).".to_string()];
        // }
        let mut ls_errors: Vec<String> = Vec::new();
        let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
        //-- do children have the parent too?
        for key in cos.keys() {
            let co = cos.get(key).unwrap().as_object().unwrap();
            if co.contains_key("children") {
                let thechildrenkeys = co.get("children").unwrap().as_array().unwrap();
                for ckey in thechildrenkeys {
                    if !cos.contains_key(ckey.as_str().unwrap()) {
                        let s =
                            format!("CityObject #{} doesn't exit [referenced by #{}]", ckey, key);
                        ls_errors.push(s);
                    }
                }
                for ckey in thechildrenkeys {
                    if !cos.contains_key(ckey.as_str().unwrap()) {
                        let s =
                            format!("CityObject #{} doesn't exit (referenced by #{}", ckey, key);
                        ls_errors.push(s);
                    } else {
                        if (!cos
                            .get(ckey.as_str().unwrap())
                            .unwrap()
                            .as_object()
                            .unwrap()
                            .contains_key("parents"))
                            || (!cos
                                .get(ckey.as_str().unwrap())
                                .unwrap()
                                .as_object()
                                .unwrap()
                                .get("parents")
                                .unwrap()
                                .as_array()
                                .unwrap()
                                .contains(&json!(key)))
                        {
                            let s = format!(
                                "CityObject #{} doesn't reference correct parent ({})",
                                ckey, key
                            );
                            ls_errors.push(s);
                        }
                    }
                }
            }
        }
        //-- are there orphans?
        for key in cos.keys() {
            let co = cos.get(key).unwrap().as_object().unwrap();
            if co.contains_key("parents") {
                let theparentkeys = co.get("parents").unwrap().as_array().unwrap();
                for pkey in theparentkeys {
                    if !cos.contains_key(pkey.as_str().unwrap()) {
                        let s = format!(
                            "CityObject #{} is an orphan [parent #{} doesn't exist]",
                            key, pkey
                        );
                        ls_errors.push(s);
                    }
                }
            }
        }
        return ls_errors;
    }

    pub fn duplicate_vertices(&self) -> Vec<String> {
        let mut ls_errors: Vec<String> = Vec::new();
        let vs = self.j.get("vertices").unwrap().as_array().unwrap();
        // use all vertices as keys in a hashmap
        let mut uniques = HashSet::new();
        for i in 0..vs.len() {
            let v = vs[i].as_array().unwrap();
            let s: String = format!(
                "{}{}{}",
                v[0].to_string(),
                v[1].to_string(),
                v[2].to_string()
            );
            if !uniques.contains(&s) {
                uniques.insert(s);
            } else {
                ls_errors.push(format!("Vertex ({}, {}, {}) duplicated", v[0], v[1], v[2]));
            }
        }
        return ls_errors;
    }
}
