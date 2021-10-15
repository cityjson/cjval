use jsonschema::{Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;

use url::Url;

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

#[derive(Serialize, Deserialize, Debug)]
struct GeomMSu {
    boundaries: Vec<Vec<Vec<usize>>>,
}
#[derive(Serialize, Deserialize, Debug)]
struct GeomSol {
    boundaries: Vec<Vec<Vec<Vec<usize>>>>,
}
#[derive(Serialize, Deserialize, Debug)]
struct GeomMSol {
    boundaries: Vec<Vec<Vec<Vec<Vec<usize>>>>>,
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
    jexts: Vec<Value>,
    duplicate_keys: bool,
}

impl CJValidator {
    pub fn from_str(s: &str) -> Result<Self, String> {
        let l: Vec<Value> = Vec::new();
        let mut v = CJValidator {
            j: json!(null),
            jexts: l,
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
        } else if self.j["version"] == "1.0" {
            version = 10;
        } else {
            let s: String = format!(
                "CityJSON version {} not supported [only \"1.0\" and \"1.1\"]",
                self.j["version"]
            );
            return vec![s];
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

    /// Does the CityJSON contain Extension(s)?
    pub fn contains_extensions(&self) -> Vec<String> {
        let mut re: Vec<String> = Vec::new();
        let v = self.j.as_object().unwrap();
        if v.contains_key("extensions") {
            let exts = self.j.get("extensions").unwrap().as_object().unwrap();
            for key in exts.keys() {
                let u = Url::parse(exts[key]["url"].as_str().unwrap()).unwrap();
                let l = u.path_segments().map(|c| c.collect::<Vec<_>>()).unwrap();
                re.push(l.last().unwrap().to_string());
            }
        }
        re
    }

    pub fn add_extension(&mut self, s: &str) -> bool {
        let re: Result<Value, _> = serde_json::from_str(&s);
        if re.is_err() {
            return false;
        }
        self.jexts.push(re.unwrap());
        true
    }

    fn validate_ext_co(&self, jext: &Value) -> Vec<String> {
        //-- 1. build the schema file from the Extension file

        let v = jext.get("extraCityObjects").unwrap().as_object().unwrap();
        for key in v.keys() {
            println!("==>{:?}", key);
            let mut schema = jext["extraCityObjects"][key].clone();
            schema["$schema"] = json!("http://json-schema.org/draft-07/schema#");
            schema["$id"] = json!("https://www.cityjson.org/schemas/1.1.0/tmp.json");
            let s1 = std::fs::read_to_string(
                "/Users/hugo/projects/cjval2/schemas/11/cityobjects.schema.json",
            )
            .expect("Couldn't read CityJSON file");
            let schema1 = serde_json::from_str(&s1).unwrap();
            let s2 = std::fs::read_to_string(
                "/Users/hugo/projects/cjval2/schemas/11/geomprimitives.schema.json",
            )
            .expect("Couldn't read CityJSON file");
            let schema2 = serde_json::from_str(&s2).unwrap();

            let compiled = JSONSchema::options()
                .with_draft(Draft::Draft7)
                .with_document(
                    "https://www.cityjson.org/schemas/1.1.0/cityobjects.schema.json".to_string(),
                    schema1,
                )
                .with_document(
                    "https://www.cityjson.org/schemas/1.1.0/geomprimitives.schema.json".to_string(),
                    schema2,
                )
                .compile(&schema)
                .expect("A valid schema");

            // println!("{:?}", compiled);
            //-- 2. fetch the CO

            let result = compiled.validate(&self.j["CityObjects"]["id-1"]);
            if result.is_ok() {
                println!("VaLiD!!!");
            } else {
                if let Err(errors) = result {
                    for error in errors {
                        println!("Validation error: {}", error);
                        println!("Instance path: {}", error.instance_path);
                    }
                }
            }
        }

        vec![]
    }

    pub fn validate_extensions(&self) -> Vec<String> {
        for ext in &self.jexts {
            // println!("{:?}", ext);
            //-- 1. extraCityObjects
            self.validate_ext_co(&ext);
        }
        vec![]
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

    pub fn wrong_vertex_index(&self) -> Vec<String> {
        let max_index: usize = self.j.get("vertices").unwrap().as_array().unwrap().len();
        let mut ls_errors: Vec<String> = Vec::new();
        let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
        for key in cos.keys() {
            let x = self.j["CityObjects"][key]["geometry"].as_array();
            if x.is_some() {
                for g in x.unwrap() {
                    if g["type"] == "MultiSurface" || g["type"] == "CompositeSurface" {
                        let aa: GeomMSu = serde_json::from_value(g.clone()).unwrap();
                        let re = above_max_index_msur(&aa.boundaries, max_index);
                        if re.is_err() {
                            ls_errors.push(re.err().unwrap());
                        }
                    } else if g["type"] == "Solid" {
                        let aa: GeomSol = serde_json::from_value(g.clone()).unwrap();
                        let re = above_max_index_sol(&aa.boundaries, max_index);
                        if re.is_err() {
                            ls_errors.push(re.err().unwrap());
                        }
                    } else if g["type"] == "MultiSolid" || g["type"] == "CompositeSolid" {
                        let aa: GeomMSol = serde_json::from_value(g.clone()).unwrap();
                        let re = above_max_index_msol(&aa.boundaries, max_index);
                        if re.is_err() {
                            ls_errors.push(re.err().unwrap());
                        }
                    }
                }
            }
        }
        ls_errors
    }
}

fn above_max_index_msur(a: &Vec<Vec<Vec<usize>>>, max_index: usize) -> Result<(), String> {
    let mut r: Vec<usize> = vec![];
    for x in a {
        for y in x {
            for z in y {
                if z >= &max_index {
                    r.push(*z);
                }
            }
        }
    }
    if r.is_empty() {
        return Ok(());
    } else {
        let mut s: String = "".to_string();
        for each in r {
            s += "#";
            s += &each.to_string();
            s += "/";
        }
        let s2 = format!("Vertices {} don't exist", s);
        return Err(s2);
    }
}

fn above_max_index_sol(a: &Vec<Vec<Vec<Vec<usize>>>>, max_index: usize) -> Result<(), String> {
    let mut r: Vec<usize> = vec![];
    for x in a {
        for y in x {
            for z in y {
                for w in z {
                    if w >= &max_index {
                        r.push(*w);
                    }
                }
            }
        }
    }
    if r.is_empty() {
        return Ok(());
    } else {
        let mut s: String = "".to_string();
        for each in r {
            s += "#";
            s += &each.to_string();
            s += "/";
        }
        let s2 = format!("Vertices {} don't exist", s);
        return Err(s2);
    }
}

fn above_max_index_msol(
    a: &Vec<Vec<Vec<Vec<Vec<usize>>>>>,
    max_index: usize,
) -> Result<(), String> {
    let mut r: Vec<usize> = vec![];
    for x in a {
        for y in x {
            for z in y {
                for w in z {
                    for q in w {
                        if q >= &max_index {
                            r.push(*q);
                        }
                    }
                }
            }
        }
    }
    if r.is_empty() {
        return Ok(());
    } else {
        let mut s: String = "".to_string();
        for each in r {
            s += "#";
            s += &each.to_string();
            s += "/";
        }
        let s2 = format!("Vertices {} don't exist", s);
        return Err(s2);
    }
}
