use jsonschema::{Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;

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

    pub fn validate_schema(&mut self) -> Vec<String> {
        self.is_schema_valid = Some(false);
        if self.j.is_null() {
            return vec!["Not a valid JSON file".to_string()];
        }
        if self.is_cityjson.is_none() {
            println!("here");
            return vec!["Not a CityJSON file".to_string()];
        }
        if self.version.is_none() {
            return vec!["Not a supported CityJSON version (v1.0 && v1.1)".to_string()];
        }
        if (self.version.unwrap() < 10) || (self.version.unwrap() > 11) {
            return vec!["CityJSON version not supported".to_string()];
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
            return ls_errors;
        } else {
            self.is_schema_valid = Some(true);
            return vec![];
        }
    }

    // parent_children_consistency
    pub fn parent_children_consistency(&self) -> Vec<String> {
        if (self.is_schema_valid.is_none()) || (self.is_schema_valid.unwrap() != true) {
            return vec!["This is not schema valid (or hasn't been tested yet).".to_string()];
        }
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

    // fn validate_duplicate_vertices(&self) -> bool {
    //     let mut valid = true;
    //     if self.version.unwrap() == 10 {
    //         let vs: Vec<VertexF> = serde_json::from_value(j["vertices"].take()).unwrap();
    //     }
    //     let verts = self
    //         .j
    //         .get("vertices")
    //         .expect("no vertices")
    //         .as_array()
    //         .expect("not an array");
    //     // use all vertices as keys in a hashmap
    //     let mut uniques = HashMap::new();
    //     for i in 0..verts.len() {
    //         let vert = verts[i].as_array().unwrap();
    //         let arr = [
    //             vert[0].to_string(),
    //             vert[1].to_string(),
    //             vert[2].to_string(),
    //         ];
    //         if !uniques.contains_key(&arr) {
    //             uniques.insert(arr, i);
    //         } else {
    //             // duplicate found!
    //             let other = uniques.get(&arr).unwrap();
    //             valid = false;
    //             // // feedback
    //             // plog!("");
    //             // plog!("Duplicate Vertex Error");
    //             // plog!("  L indices : vertices[{}] == vertices[{}]", other, i);
    //             // plog!("  L vertex  : [{}, {}, {}]", arr[0], arr[1], arr[2]);
    //         }
    //     }
    //     return valid;
    // }
}
