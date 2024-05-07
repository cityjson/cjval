//! # cjval: a validator for CityJSON
//!
//! A library to validate the syntax of CityJSON objects (CityJSON +
//! [CityJSONFeatures](https://www.cityjson.org/specs/#text-sequences-and-streaming-with-cityjsonfeature)).
//!
//! It validates against the [CityJSON schemas](https://www.cityjson.org/schemas) and additional functions have been implemented
//! (because these can't be expressed with [JSON Schema](https://json-schema.org/)).
//!
//! The following is error checks are performed:
//!
//!   1. *JSON syntax*: is the file a valid JSON file?
//!   1. *CityJSON schemas*: validation against the schemas (CityJSON v1.0 or v1.1)
//!   1. *Extension schemas*: validate against the extra schemas if there's an Extension in the input file
//!   1. *parents_children_consistency*: if a City Object references another in its `children`, this ensures that the child exists. And that the child has the parent in its `parents`
//!   1. *wrong_vertex_index*: checks if all vertex indices exist in the list of vertices
//!   1. *semantics_array*: checks if the arrays for the semantics in the geometries have the same shape as that of the geometry and if the values are consistent
//!   1. *textures*: checks if the arrays for the textures are coherent (if the vertices exist + if the texture linked to exists)
//!   1. *materials*: checks if the arrays for the materials are coherent with the geometry objects and if material linked to exists

//!
//! It also verifies the following, these are not errors since the file is still considered valid and usable, but they can make the file larger and some parsers might not understand all the properties:
//!
//!   1. *extra_root_properties*: if CityJSON has extra root properties, these should be documented in an Extension. If not this warning is returned
//!   1. *duplicate_vertices*: duplicated vertices in `vertices` are allowed, but they take up spaces and decreases the topological relationships explicitly in the file. If there are any, [cjio](https://github.com/cityjson/cjio) has the operator `clean` to fix this automatically.
//!   1. *unused_vertices*: vertices that are not referenced in the file, they take extra space. If there are any, [cjio](https://github.com/cityjson/cjio) has the operator `clean` to fix this automatically.
//!
//! ## Library + 3 binaries
//!
//! `cjval` is a library and has 3 different binaries:
//!
//!   1. `cjval` to validate a CityJSON file (it downloads automatically Extensions)
//!   2. `cjfval` to validate a stream of CityJSONFeature (from stdin)
//!   3. `cjvalext` to validate a [CityJSON Extension file](https://www.cityjson.org/specs/#the-extension-file)
//!
//!
//! ## Example use
//!
//! ```rust
//! extern crate cjval;
//!
//! fn main() {
//!     let s1 = std::fs::read_to_string("/Users/hugo/projects/cjval/data/cube.city.json")
//!         .expect("Couldn't read CityJSON file");
//!     let v = cjval::CJValidator::from_str(&s1);
//!     let re = v.validate();
//!     for (criterion, sum) in re.iter() {
//!         println!("=== {} ===", criterion);
//!         println!("{}", sum);
//!     }
//! }
//! ```
//!
//! ## Installation/compilation
//!
//! ### To install the binaries on your system easily
//!
//! 1. install the [Rust compiler](https://www.rust-lang.org/learn/get-started)
//! 2. `cargo install cjval --features build-binary`
//!
//!

use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use jsonschema::{Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;

// #-- ERRORS
//  # schema
//  # extensions
//  # parents_children_consistency
//  # wrong_vertex_index
//  # semantics_arrays
//  # textures
//  # materials
//
// #-- WARNINGS
//  # extra_root_properties
//  # duplicate_vertices
//  # unused_vertices

static EXTENSION_FIXED_NAMES: [&str; 6] = [
    "type",
    "name",
    "url",
    "version",
    "versionCityJSON",
    "description",
];

/// Summary of a validation. It is possible that a validation check has not
/// been performed because other checks returned errors (we do not want to
/// have cascading errors).
#[derive(Debug)]
pub struct ValSummary {
    status: Option<bool>,
    errors: Vec<String>,
    warning: bool,
}

impl ValSummary {
    fn new() -> ValSummary {
        let l: Vec<String> = Vec::new();
        ValSummary {
            status: None,
            errors: l,
            warning: false,
        }
    }
    fn set_validity(&mut self, b: bool) {
        self.status = Some(b);
    }
    fn set_as_warning(&mut self) {
        self.warning = true;
    }
    /// Returns true if it's a warning (and not an error)
    pub fn is_warning(&self) -> bool {
        self.warning
    }
    /// Returns true if valid, false if not (and also false if not performed)
    pub fn is_valid(&self) -> bool {
        if self.status == Some(true) {
            return true;
        } else {
            return false;
        }
    }
    /// Returns true if errors are present
    pub fn has_errors(&self) -> bool {
        match self.status {
            Some(s) => {
                if s == true {
                    return false;
                } else {
                    return true;
                }
            }
            None => return false,
        }
    }
    fn add_error(&mut self, e: String) {
        self.errors.push(e);
        self.set_validity(false);
    }
}

impl fmt::Display for ValSummary {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.status {
            Some(s) => {
                if s == true {
                    fmt.write_str("ok")?;
                } else {
                    fmt.write_str(&format!("{}", self.errors.join("\n")))?;
                }
            }
            None => (),
        }
        Ok(())
    }
}

static CITYJSON_V10_VERSION: &str = "1.0.3";

#[derive(Serialize, Deserialize, Debug)]
struct GeomMPo {
    boundaries: Vec<usize>,
}
#[derive(Serialize, Deserialize, Debug)]
struct GeomMLS {
    boundaries: Vec<Vec<usize>>,
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
#[derive(Serialize, Deserialize, Debug)]
struct TextureMSu {
    values: Vec<Vec<Vec<Option<usize>>>>,
}
#[derive(Serialize, Deserialize, Debug)]
struct TextureSol {
    values: Vec<Vec<Vec<Vec<Option<usize>>>>>,
}
#[derive(Serialize, Deserialize, Debug)]
struct TextureMSol {
    values: Vec<Vec<Vec<Vec<Vec<Option<usize>>>>>>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, PartialEq)]
struct Doc {
    #[serde(with = "::serde_with::rust::maps_duplicate_key_is_error")]
    CityObjects: HashMap<String, Value>,
}

pub fn get_cityjson_schema_all_versions() -> Vec<String> {
    let mut l: Vec<String> = Vec::new();
    //-- v1.0
    l.push(CITYJSON_V10_VERSION.to_string());
    //-- v1.1
    let schema_str = include_str!("../schemas/11/cityjson.min.schema.json");
    let schema: Value = serde_json::from_str(schema_str).unwrap();
    let vs = &schema["$id"].to_string();
    l.push(vs.get(34..39).unwrap().to_string());
    //-- v2.0
    let schema_str = include_str!("../schemas/20/cityjson.min.schema.json");
    let schema: Value = serde_json::from_str(schema_str).unwrap();
    let vs = &schema["$id"].to_string();
    l.push(vs.get(34..39).unwrap().to_string());
    l
}

/// A validator for CityJSON and CityJSONFeature
#[derive(Debug)]
pub struct CJValidator {
    j: Value,
    jschema_cj: Value,
    jschema_cjf: Value,
    jexts: Vec<Value>,
    json_syntax_error: Option<String>,
    duplicate_keys: bool,
    is_cityjson: bool,
    is_cjfeature: bool,
    version_file: i32,
    version_schema: String,
}

impl CJValidator {
    /// Creates a CJValidator from a &str.
    /// Will not return an error here if the &str is not a JSON,
    /// only when validate() is called can you see that error.
    /// ```rust
    /// use cjval::CJValidator;
    /// let s1 = std::fs::read_to_string("./data/cube.city.json")
    ///         .expect("Couldn't read CityJSON file");
    /// let v = CJValidator::from_str(&s1);
    /// ```
    pub fn from_str(str_dataset: &str) -> Self {
        let l: Vec<Value> = Vec::new();
        let mut v = CJValidator {
            j: json!(null),
            jschema_cj: json!(null),
            jschema_cjf: json!(null),
            jexts: l,
            json_syntax_error: None,
            duplicate_keys: false,
            is_cityjson: true,
            is_cjfeature: false,
            version_file: 0,
            version_schema: "-1".to_string(),
        };
        //-- parse the dataset and convert to JSON
        let re = serde_json::from_str(&str_dataset);
        match re {
            Ok(j) => {
                v.j = j;
                // TODO: what if j.is_null() is true?
            }
            Err(e) => v.json_syntax_error = Some(e.to_string()),
        }
        //-- check the type
        if v.j["type"] == "CityJSON" {
            //-- check cityjson version
            if v.j["version"] == "2.0" {
                v.version_file = 20;
                let schema_str = include_str!("../schemas/20/cityjson.min.schema.json");
                v.jschema_cj = serde_json::from_str(schema_str).unwrap();
                let vs = &v.jschema_cj["$id"].to_string();
                v.version_schema = vs.get(34..39).unwrap().to_string();
                //-- for CityJSONFeature
                let schemaf_str = include_str!("../schemas/20/cityjsonfeature.min.schema.json");
                v.jschema_cjf = serde_json::from_str(schemaf_str).unwrap();
            } else if v.j["version"] == "1.1" {
                v.version_file = 11;
                let schema_str = include_str!("../schemas/11/cityjson.min.schema.json");
                v.jschema_cj = serde_json::from_str(schema_str).unwrap();
                let vs = &v.jschema_cj["$id"].to_string();
                v.version_schema = vs.get(34..39).unwrap().to_string();
                //-- for CityJSONFeature
                let schemaf_str = include_str!("../schemas/11/cityjsonfeature.min.schema.json");
                v.jschema_cjf = serde_json::from_str(schemaf_str).unwrap();
            } else if v.j["version"] == "1.0" {
                v.version_file = 10;
                let schema_str = include_str!("../schemas/10/cityjson.min.schema.json");
                v.jschema_cj = serde_json::from_str(schema_str).unwrap();
                v.version_schema = "1.0.3".to_string();
            }
        } else {
            v.is_cityjson = false;
        }
        //-- check for duplicate keys in CO object, Doc is the struct above
        //-- used for identifying duplicate keys
        if v.json_syntax_error.is_none() {
            let re: Result<Doc, _> = serde_json::from_str(&str_dataset);
            if re.is_err() {
                v.duplicate_keys = true;
            }
        }
        v
    }

    pub fn from_str_cjfeature(&mut self, str_cjf: &str) -> Result<(), String> {
        //-- parse the cjf and convert to JSON
        let re: Result<Value, _> = serde_json::from_str(&str_cjf);
        if re.is_err() {
            return Err(re.err().unwrap().to_string());
        }
        let j: Value = re.unwrap();
        if j["type"] != "CityJSONFeature" {
            return Err("Not a CityJSONFeature object".to_string());
        }
        self.j = j;
        self.is_cjfeature = true;
        // println!("{:?}", self.version_file);
        // if self.version == "2.0" {
        //     v.version_file = 20;
        //     let schema_str = include_str!("../schemas/20/cityjsonfeature.min.schema.json");
        //     self.jschema = serde_json::from_str(schema_str).unwrap();
        //     let vs = &v.jschema["$id"].to_string();
        //     self.version_schema = vs.get(34..39).unwrap().to_string();
        // } else if v.version == "1.1" {
        //     self.version_file = 11;
        //     let schema_str = include_str!("../schemas/11/cityjsonfeature.min.schema.json");
        //     v.jschema = serde_json::from_str(schema_str).unwrap();
        //     let vs = &v.jschema["$id"].to_string();
        //     v.version_schema = vs.get(34..39).unwrap().to_string();
        // }

        Ok(())
    }

    /// Add the content (&str) of an Extension.
    /// The library cannot download automatically the Extensions.
    /// ```rust
    /// use cjval::CJValidator;
    /// let sdata = std::fs::read_to_string("./data/cube.city.json")
    ///         .expect("Couldn't read CityJSON file");
    /// let sext = std::fs::read_to_string("./data/generic.ext.json")
    ///         .expect("Couldn't read JSON file");
    /// let mut val = CJValidator::from_str(&sdata);
    /// let re = val.add_one_extension_from_str(&sext);
    /// ```
    pub fn add_one_extension_from_str(&mut self, ext_schema_str: &str) -> Result<()> {
        let re: Result<Value, _> = serde_json::from_str(ext_schema_str);
        if re.is_err() {
            return Err(anyhow!(re.err().unwrap().to_string()));
        }
        self.jexts.push(re.unwrap());
        Ok(())
    }

    /// Returns true if the CityJSON/Feature does not contain errors.
    /// False otherwise.
    pub fn is_valid(&self) -> bool {
        let valsumm = self.validate();
        if valsumm["json_syntax"].has_errors() {
            return false;
        }
        if valsumm["schema"].has_errors() {
            return false;
        }
        if valsumm["extensions"].has_errors() {
            return false;
        }
        if valsumm["parents_children_consistency"].has_errors() {
            return false;
        }
        if valsumm["wrong_vertex_index"].has_errors() {
            return false;
        }
        if valsumm["semantics_arrays"].has_errors() {
            return false;
        }
        if valsumm["materials"].has_errors() {
            return false;
        }
        if valsumm["textures"].has_errors() {
            return false;
        }
        true
    }

    /// The function to performs all the checks (errors+warnings).
    /// Return a IndexMap (a HashMap where keys are ordered) containing
    /// the check name and a ValSummary.
    /// ```rust
    /// use cjval::CJValidator;
    /// let s1 = std::fs::read_to_string("./data/many.json")
    ///     .expect("Couldn't read CityJSON file");
    /// let v = CJValidator::from_str(&s1);
    /// let re = v.validate();
    /// for (criterion, sum) in re.iter() {
    ///     println!("=== {} ===", criterion);
    ///     println!("{}", sum);
    /// }
    /// ```
    pub fn validate(&self) -> IndexMap<String, ValSummary> {
        let mut w1 = ValSummary::new();
        w1.set_as_warning();
        let mut w2 = ValSummary::new();
        w2.set_as_warning();
        let mut w3 = ValSummary::new();
        w3.set_as_warning();
        let mut vsum = IndexMap::from([
            ("json_syntax".to_string(), ValSummary::new()),
            ("schema".to_string(), ValSummary::new()),
            ("extensions".to_string(), ValSummary::new()),
            (
                "parents_children_consistency".to_string(),
                ValSummary::new(),
            ),
            ("wrong_vertex_index".to_string(), ValSummary::new()),
            ("semantics_arrays".to_string(), ValSummary::new()),
            ("textures".to_string(), ValSummary::new()),
            ("materials".to_string(), ValSummary::new()),
            ("extra_root_properties".to_string(), w1),
            ("duplicate_vertices".to_string(), w2),
            ("unused_vertices".to_string(), w3),
        ]);

        //-- json_syntax
        match &self.json_syntax_error {
            Some(e) => {
                vsum.get_mut("json_syntax")
                    .unwrap()
                    .add_error(e.to_string());
                return vsum;
            }
            None => vsum.get_mut("json_syntax").unwrap().set_validity(true),
        }

        //-- schema
        let mut re = self.schema();
        match re {
            Ok(_) => vsum.get_mut("schema").unwrap().set_validity(true),
            Err(errs) => {
                for err in errs {
                    vsum.get_mut("schema").unwrap().add_error(err);
                }
                return vsum;
            }
        }
        if self.duplicate_keys == true {
            vsum.get_mut("schema")
                .unwrap()
                .add_error("Duplicate keys in 'CityObjects'".to_string());
            return vsum;
        }

        //-- extensions
        re = self.validate_extensions();
        match re {
            Ok(_) => vsum.get_mut("extensions").unwrap().set_validity(true),
            Err(errs) => {
                for err in errs {
                    vsum.get_mut("extensions").unwrap().add_error(err);
                }
                return vsum;
            }
        }

        //-- parents_children_consistency
        re = self.parents_children_consistency();
        match re {
            Ok(_) => vsum
                .get_mut("parents_children_consistency")
                .unwrap()
                .set_validity(true),
            Err(errs) => {
                for err in errs {
                    vsum.get_mut("parents_children_consistency")
                        .unwrap()
                        .add_error(err);
                }
            }
        }
        //-- wrong_vertex_index
        re = self.wrong_vertex_index();
        match re {
            Ok(_) => vsum
                .get_mut("wrong_vertex_index")
                .unwrap()
                .set_validity(true),
            Err(errs) => {
                for err in errs {
                    vsum.get_mut("wrong_vertex_index").unwrap().add_error(err);
                }
            }
        }
        //-- semantics_arrays
        re = self.semantics_arrays();
        match re {
            Ok(_) => vsum.get_mut("semantics_arrays").unwrap().set_validity(true),
            Err(errs) => {
                for err in errs {
                    vsum.get_mut("semantics_arrays").unwrap().add_error(err);
                }
            }
        }
        //-- textures
        re = self.textures();
        match re {
            Ok(_) => vsum.get_mut("textures").unwrap().set_validity(true),
            Err(errs) => {
                for err in errs {
                    vsum.get_mut("textures").unwrap().add_error(err);
                }
            }
        }
        //-- materials
        re = self.materials();
        match re {
            Ok(_) => vsum.get_mut("materials").unwrap().set_validity(true),
            Err(errs) => {
                for err in errs {
                    vsum.get_mut("materials").unwrap().add_error(err);
                }
            }
        }

        //-- warnings : only do if no errors so far
        for (_c, summ) in vsum.iter() {
            if summ.has_errors() == true {
                return vsum;
            }
        }
        //-- extra_root_properties
        re = self.extra_root_properties();
        match re {
            Ok(_) => vsum
                .get_mut("extra_root_properties")
                .unwrap()
                .set_validity(true),
            Err(errs) => {
                for err in errs {
                    vsum.get_mut("extra_root_properties")
                        .unwrap()
                        .add_error(err);
                }
            }
        }
        //-- duplicate_vertices
        re = self.duplicate_vertices();
        match re {
            Ok(_) => vsum
                .get_mut("duplicate_vertices")
                .unwrap()
                .set_validity(true),
            Err(errs) => {
                for err in errs {
                    vsum.get_mut("duplicate_vertices").unwrap().add_error(err);
                }
            }
        }
        //-- unused_vertices
        re = self.unused_vertices();
        match re {
            Ok(_) => vsum.get_mut("unused_vertices").unwrap().set_validity(true),
            Err(errs) => {
                for err in errs {
                    vsum.get_mut("unused_vertices").unwrap().add_error(err);
                }
            }
        }
        return vsum;
    }

    pub fn get_extensions_urls(&self) -> Option<Vec<String>> {
        let mut re: Vec<String> = Vec::new();
        let v = self.j.as_object().unwrap();
        if v.contains_key("extensions") {
            let exts = self.j.get("extensions").unwrap().as_object().unwrap();
            for key in exts.keys() {
                re.push(exts[key]["url"].as_str().unwrap().to_string());
            }
        }
        if re.is_empty() {
            None
        } else {
            Some(re)
        }
    }

    pub fn is_cityjson(&self) -> bool {
        self.is_cityjson
    }

    pub fn is_cityjsonfeature(&self) -> bool {
        self.is_cjfeature
    }

    pub fn get_input_cityjson_version(&self) -> i32 {
        self.version_file
    }

    pub fn get_cjseq_feature_id(&self) -> String {
        if self.is_cjfeature {
            match self.j.get("id") {
                Some(x) => {
                    return x.as_str().unwrap().to_string();
                }
                None => return "".to_string(),
            }
        } else {
            return "".to_string();
        }
    }

    pub fn get_cityjson_schema_version(&self) -> String {
        self.version_schema.to_owned()
    }

    fn schema(&self) -> Result<(), Vec<String>> {
        let mut ls_errors: Vec<String> = Vec::new();
        //-- if type == CityJSON
        if self.is_cityjson == false {
            let s: String = format!("Not a CityJSON file");
            return Err(vec![s]);
        }
        if self.is_cjfeature == false {
            //-- which cityjson version
            if self.version_file == 0 {
                let s: String = format!(
                    "CityJSON version {} not supported (or missing) [only \"1.0\", \"1.1\", \"2.0\"]",
                    self.j["version"]
                );
                return Err(vec![s]);
            }
        }

        if self.is_cjfeature == false {
            let compiled = JSONSchema::options()
                .with_draft(Draft::Draft7)
                .compile(&self.jschema_cj)
                .expect("A valid schema");
            let result = compiled.validate(&self.j);
            if let Err(errors) = result {
                for error in errors {
                    let s: String = format!("{} [path:{}]", error, error.instance_path);
                    ls_errors.push(s);
                }
            }
        } else {
            let compiled = JSONSchema::options()
                .with_draft(Draft::Draft7)
                .compile(&self.jschema_cjf)
                .expect("A valid schema");
            let result = compiled.validate(&self.j);
            if let Err(errors) = result {
                for error in errors {
                    let s: String = format!("{} [path:{}]", error, error.instance_path);
                    ls_errors.push(s);
                }
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn validate_ext_extracityobjects(&self, jext: &Value) -> Result<(), Vec<String>> {
        let mut ls_errors: Vec<String> = Vec::new();
        //-- 1. build the schema file from the Extension file
        let v = jext.get("extraCityObjects").unwrap().as_object().unwrap();
        let jexto = jext.as_object().unwrap();
        for eco in v.keys() {
            // println!("==>{:?}", eco);
            let mut schema = jext["extraCityObjects"][eco].clone();
            schema["$schema"] = json!("http://json-schema.org/draft-07/schema#");
            if self.version_file == 11 {
                schema["$id"] = json!("https://www.cityjson.org/schemas/1.1.0/tmp.json");
            } else if self.version_file == 20 {
                schema["$id"] = json!("https://www.cityjson.org/schemas/2.0.0/tmp.json");
            }
            for each in jexto.keys() {
                let ss = each.as_str();
                if EXTENSION_FIXED_NAMES.contains(&ss) == false {
                    schema[ss] = jext[ss].clone();
                }
            }
            // println!("=>{}", serde_json::to_string(&schema).unwrap());
            let compiled = self.get_compiled_schema_extension(&schema).unwrap();
            //-- 2. fetch the CO
            let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
            for co in cos.keys() {
                let tmp = cos.get(co).unwrap().as_object().unwrap();
                if tmp["type"].as_str().unwrap() == eco {
                    // println!("here");
                    let result = compiled.validate(&self.j["CityObjects"][co]);
                    if let Err(errors) = result {
                        for error in errors {
                            let s: String = format!("{} [path:{}]", error, error.instance_path);
                            ls_errors.push(s);
                        }
                    }
                }
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn validate_ext_extrarootproperties(&self, jext: &Value) -> Result<(), Vec<String>> {
        let mut ls_errors: Vec<String> = Vec::new();
        //-- 1. build the schema file from the Extension file
        let v = jext
            .get("extraRootProperties")
            .unwrap()
            .as_object()
            .unwrap();
        let jexto = jext.as_object().unwrap();
        for rp in v.keys() {
            // println!("==>{:?}", eco);
            let mut schema = jext["extraRootProperties"][rp].clone();
            schema["$schema"] = json!("http://json-schema.org/draft-07/schema#");
            if self.version_file == 11 {
                schema["$id"] = json!("https://www.cityjson.org/schemas/1.1.0/tmp.json");
            } else if self.version_file == 20 {
                schema["$id"] = json!("https://www.cityjson.org/schemas/2.0.0/tmp.json");
            }
            for each in jexto.keys() {
                let ss = each.as_str();
                if EXTENSION_FIXED_NAMES.contains(&ss) == false {
                    schema[ss] = jext[ss].clone();
                }
            }
            let compiled = self.get_compiled_schema_extension(&schema).unwrap();

            for k in self.j.as_object().unwrap().keys() {
                if k == rp {
                    let result = compiled.validate(&self.j[k]);
                    if let Err(errors) = result {
                        for error in errors {
                            let s: String = format!("{} [path:{}]", error, error.instance_path);
                            ls_errors.push(s);
                        }
                    }
                }
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn validate_ext_extraattributes(&self, jext: &Value) -> Result<(), Vec<String>> {
        let mut ls_errors: Vec<String> = Vec::new();
        //-- 1. build the schema file from the Extension file
        let v = jext.get("extraAttributes").unwrap().as_object().unwrap();
        let jexto = jext.as_object().unwrap();
        for cotype in v.keys() {
            //-- for each CityObject type
            for eatt in jext["extraAttributes"][cotype].as_object().unwrap().keys() {
                let mut schema = jext["extraAttributes"][cotype][eatt.as_str()].clone();
                schema["$schema"] = json!("http://json-schema.org/draft-07/schema#");
                if self.version_file == 11 {
                    schema["$id"] = json!("https://www.cityjson.org/schemas/1.1.0/tmp.json");
                } else if self.version_file == 20 {
                    schema["$id"] = json!("https://www.cityjson.org/schemas/2.0.0/tmp.json");
                }
                for each in jexto.keys() {
                    let ss = each.as_str();
                    if EXTENSION_FIXED_NAMES.contains(&ss) == false {
                        schema[ss] = jext[ss].clone();
                    }
                }
                let compiled = self.get_compiled_schema_extension(&schema).unwrap();
                let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
                for oneco in cos.keys() {
                    let tmp = cos.get(oneco).unwrap().as_object().unwrap();
                    if tmp["type"].as_str().unwrap() == cotype
                        && tmp.contains_key("attributes")
                        && tmp["attributes"].as_object().unwrap().contains_key(eatt)
                    {
                        let result =
                            compiled.validate(&self.j["CityObjects"][oneco]["attributes"][eatt]);
                        if let Err(errors) = result {
                            for error in errors {
                                let s: String = format!("{} [path:{}]", error, error.instance_path);
                                ls_errors.push(s);
                            }
                        }
                    }
                }
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn validate_ext_extrasemanticsurfaces(&self, jext: &Value) -> Result<(), Vec<String>> {
        let mut ls_errors: Vec<String> = Vec::new();
        //-- 0. check if "extraSemanticSurfaces" is in the file, if not then all good
        let t = jext["extraSemanticSurfaces"].as_object();
        if t.is_none() {
            return Ok(());
        }
        //-- 1. build the schema file from the Extension file
        let v = jext
            .get("extraSemanticSurfaces")
            .unwrap()
            .as_object()
            .unwrap();
        let jexto = jext.as_object().unwrap();
        for semsurf in v.keys() {
            let mut schema = jext["extraSemanticSurfaces"][semsurf].clone();
            schema["$schema"] = json!("http://json-schema.org/draft-07/schema#");
            if self.version_file == 11 {
                schema["$id"] = json!("https://www.cityjson.org/schemas/1.1.0/tmp.json");
            } else if self.version_file == 20 {
                schema["$id"] = json!("https://www.cityjson.org/schemas/2.0.0/tmp.json");
            }
            for each in jexto.keys() {
                let ss = each.as_str();
                if EXTENSION_FIXED_NAMES.contains(&ss) == false {
                    schema[ss] = jext[ss].clone();
                }
            }
            let compiled = self.get_compiled_schema_extension(&schema).unwrap();
            let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
            for key in cos.keys() {
                //-- check geometry
                let x = self.j["CityObjects"][key]["geometry"].as_array();
                if x.is_some() {
                    for (i, g) in x.unwrap().iter().enumerate() {
                        let surfs = g["semantics"]["surfaces"].as_array();
                        if surfs.is_some() {
                            for (j, surf) in surfs.unwrap().iter().enumerate() {
                                let tmp = surf.as_object().unwrap();
                                if tmp["type"].as_str().unwrap() == semsurf {
                                    let result = compiled.validate(
                                        &self.j["CityObjects"][key]["geometry"][i]["semantics"]
                                            ["surfaces"][j],
                                    );
                                    if let Err(errors) = result {
                                        for error in errors {
                                            let s: String =
                                                format!("{} [path:{}]", error, error.instance_path);
                                            ls_errors.push(s);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn get_compiled_schema_extension(&self, schema: &Value) -> Option<JSONSchema> {
        if self.version_file == 11 {
            let s_1 = include_str!("../schemas/11/cityobjects.schema.json");
            let s_2 = include_str!("../schemas/11/geomprimitives.schema.json");
            let s_3 = include_str!("../schemas/11/appearance.schema.json");
            let s_4 = include_str!("../schemas/11/geomtemplates.schema.json");
            let schema_1 = serde_json::from_str(s_1).unwrap();
            let schema_2 = serde_json::from_str(s_2).unwrap();
            let schema_3 = serde_json::from_str(s_3).unwrap();
            let schema_4 = serde_json::from_str(s_4).unwrap();
            let compiled = JSONSchema::options()
                .with_draft(Draft::Draft7)
                .with_document(
                    "https://www.cityjson.org/schemas/1.1.0/cityobjects.schema.json".to_string(),
                    schema_1,
                )
                .with_document(
                    "https://www.cityjson.org/schemas/1.1.0/geomprimitives.schema.json".to_string(),
                    schema_2,
                )
                .with_document(
                    "https://www.cityjson.org/schemas/1.1.0/appearance.schema.json".to_string(),
                    schema_3,
                )
                .with_document(
                    "https://www.cityjson.org/schemas/1.1.0/geomtemplates.schema.json".to_string(),
                    schema_4,
                )
                .compile(&schema)
                .expect("A valid schema");
            return Some(compiled);
        } else if self.version_file == 20 {
            let s_1 = include_str!("../schemas/20/cityobjects.schema.json");
            let s_2 = include_str!("../schemas/20/geomprimitives.schema.json");
            let s_3 = include_str!("../schemas/20/appearance.schema.json");
            let s_4 = include_str!("../schemas/20/geomtemplates.schema.json");
            let schema_1 = serde_json::from_str(s_1).unwrap();
            let schema_2 = serde_json::from_str(s_2).unwrap();
            let schema_3 = serde_json::from_str(s_3).unwrap();
            let schema_4 = serde_json::from_str(s_4).unwrap();
            let compiled = JSONSchema::options()
                .with_draft(Draft::Draft7)
                .with_document(
                    "https://www.cityjson.org/schemas/2.0.0/cityobjects.schema.json".to_string(),
                    schema_1,
                )
                .with_document(
                    "https://www.cityjson.org/schemas/2.0.0/geomprimitives.schema.json".to_string(),
                    schema_2,
                )
                .with_document(
                    "https://www.cityjson.org/schemas/2.0.0/appearance.schema.json".to_string(),
                    schema_3,
                )
                .with_document(
                    "https://www.cityjson.org/schemas/2.0.0/geomtemplates.schema.json".to_string(),
                    schema_4,
                )
                .compile(&schema)
                .expect("A valid schema");
            return Some(compiled);
        } else {
            return None;
        }
    }

    fn validate_extensions(&self) -> Result<(), Vec<String>> {
        let mut ls_errors: Vec<String> = Vec::new();
        for ext in &self.jexts {
            //-- 0. check the version of CityJSON
            let mut v: String = self.version_file.to_string();
            v.insert(1, '.');
            if ext["versionCityJSON"] != v {
                let s: String = format!(
                    "Extension 'versionCityJSON' != CityJSON version of file [{} != {}]",
                    ext["versionCityJSON"].as_str().unwrap(),
                    v
                );
                ls_errors.push(s);
            }
            //-- 1. extraCityObjects
            let mut re = self.validate_ext_extracityobjects(&ext);
            if re.is_err() {
                ls_errors.append(&mut re.err().unwrap());
            }
            //-- 2. extraRootProperties
            re = self.validate_ext_extrarootproperties(&ext);
            if re.is_err() {
                ls_errors.append(&mut re.err().unwrap());
            }
            //-- 3. extraAttributes
            re = self.validate_ext_extraattributes(&ext);
            if re.is_err() {
                ls_errors.append(&mut re.err().unwrap());
            }
            if self.version_file >= 20 {
                //-- 4. extraSemanticSurfaces
                re = self.validate_ext_extrasemanticsurfaces(&ext);
                if re.is_err() {
                    ls_errors.append(&mut re.err().unwrap());
                }
            }
        }
        //-- 5. check if there are CityObjects that do not have a schema
        let mut re = self.validate_ext_co_without_schema();
        if re.is_err() {
            ls_errors.append(&mut re.err().unwrap());
        }
        //-- 6. check if there are extra root properties that do not have a schema
        re = self.validate_ext_rootproperty_without_schema();
        if re.is_err() {
            ls_errors.append(&mut re.err().unwrap());
        }
        //-- 7. check for the extra attributes w/o schemas
        re = self.validate_ext_attribute_without_schema();
        if re.is_err() {
            ls_errors.append(&mut re.err().unwrap());
        }
        //-- 8. check for the semsurfs w/o schemas
        if self.version_file >= 20 {
            re = self.validate_ext_semsurf_without_schema();
            if re.is_err() {
                ls_errors.append(&mut re.err().unwrap());
            }
        }

        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn validate_ext_semsurf_without_schema(&self) -> Result<(), Vec<String>> {
        let mut ls_errors: Vec<String> = Vec::new();
        let mut newss: Vec<String> = Vec::new();
        for jext in &self.jexts {
            let re = jext.get("extraSemanticSurfaces");
            if re.is_some() {
                let v = re.unwrap().as_object().unwrap();
                for ess in v.keys() {
                    newss.push(ess.to_string());
                }
            }
        }
        //-- fetch the COs
        let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
        for key in cos.keys() {
            let x = self.j["CityObjects"][key]["geometry"].as_array();
            if x.is_some() {
                for g in x.unwrap() {
                    let surfs = g["semantics"]["surfaces"].as_array();
                    if surfs.is_some() {
                        for surf in surfs.unwrap() {
                            let tmp = surf.as_object().unwrap();
                            let thetype = tmp["type"].as_str().unwrap().to_string();
                            if &thetype[0..1] == "+" && newss.contains(&thetype) == false {
                                let s: String =
                                    format!("Semantic Surface '{}' doesn't have a schema", thetype);
                                ls_errors.push(s);
                            }
                        }
                    }
                }
            }
        }

        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn validate_ext_attribute_without_schema(&self) -> Result<(), Vec<String>> {
        let mut ls_errors: Vec<String> = Vec::new();
        let mut ls_plusattrs: HashSet<String> = HashSet::new();
        let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
        for theid in cos.keys() {
            let co = cos.get(theid).unwrap().as_object().unwrap();
            if co.contains_key("attributes") {
                let attrs = co.get("attributes").unwrap().as_object().unwrap();
                for attr in attrs.keys() {
                    let sattr = attr.as_str();
                    if &sattr[0..1] == "+" {
                        // println!("attr: {:?}", sattr);
                        let a = format!("{}/{}", co.get("type").unwrap().as_str().unwrap(), sattr);
                        ls_plusattrs.insert(a);
                    }
                }
            }
        }
        // println!("{:?}", ls_plusattrs);
        for each in ls_plusattrs {
            for jext in &self.jexts {
                let s = format!("/extraAttributes/{}", each);
                let re = jext.pointer(s.as_str());
                if re.is_none() {
                    let s: String = format!("Attribute '{}' doesn't have a schema", each);
                    ls_errors.push(s);
                }
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn validate_ext_co_without_schema(&self) -> Result<(), Vec<String>> {
        let mut ls_errors: Vec<String> = Vec::new();
        let mut newcos: Vec<String> = Vec::new();
        for jext in &self.jexts {
            let v = jext.get("extraCityObjects").unwrap().as_object().unwrap();
            for eco in v.keys() {
                newcos.push(eco.to_string());
            }
        }
        //-- fetch the COs
        let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
        for co in cos.keys() {
            let tmp = cos.get(co).unwrap().as_object().unwrap();
            let thetype = tmp["type"].as_str().unwrap().to_string();
            if &thetype[0..1] == "+" && newcos.contains(&thetype) == false {
                let s: String = format!("CityObject '{}' doesn't have a schema", thetype);
                ls_errors.push(s);
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn validate_ext_rootproperty_without_schema(&self) -> Result<(), Vec<String>> {
        let mut ls_errors: Vec<String> = Vec::new();
        let mut newrps: Vec<String> = Vec::new();
        for jext in &self.jexts {
            let v = jext
                .get("extraRootProperties")
                .unwrap()
                .as_object()
                .unwrap();
            for erp in v.keys() {
                newrps.push(erp.to_string());
            }
        }
        let t = self.j.as_object().unwrap();
        for each in t.keys() {
            let s = each.to_string();
            if &s[0..1] == "+" && (newrps.contains(&s) == false) {
                let s: String = format!("Extra root property '{}' doesn't have a schema", s);
                ls_errors.push(s);
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn extra_root_properties(&self) -> Result<(), Vec<String>> {
        if self.is_cjfeature {
            return Ok(());
        };
        let mut ls_warnings: Vec<String> = Vec::new();
        let rootproperties: [&str; 9] = [
            "type",
            "version",
            "extensions",
            "transform",
            "metadata",
            "CityObjects",
            "vertices",
            "appearance",
            "geometry-templates",
        ];
        let t = self.j.as_object().unwrap();
        for each in t.keys() {
            let s = each.to_string();
            if &s[0..1] != "+" && (rootproperties.contains(&s.as_str()) == false) {
                let s: String = format!("Root property '{}' is not in CityJSON schema, might be ignored by some parsers", s);
                ls_warnings.push(s);
            }
        }
        if ls_warnings.is_empty() {
            Ok(())
        } else {
            Err(ls_warnings)
        }
    }

    // parents_children_consistency
    fn parents_children_consistency(&self) -> Result<(), Vec<String>> {
        let mut ls_errors: Vec<String> = Vec::new();
        let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
        //-- do children have the parent too?
        for key in cos.keys() {
            let co = cos.get(key).unwrap().as_object().unwrap();
            if co.contains_key("children") {
                let thechildrenkeys = co.get("children").unwrap().as_array().unwrap();
                for ckey in thechildrenkeys {
                    if !cos.contains_key(ckey.as_str().unwrap()) {
                        let s = format!(
                            "CityObject #{} doesn't exist (referenced by #{})",
                            ckey.as_str().unwrap(),
                            key
                        );
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
                                "CityObject #{} doesn't reference correct parent (#{})",
                                ckey.as_str().unwrap(),
                                key
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
                            "CityObject #{} is an orphan (parent #{} doesn't exist)",
                            key,
                            pkey.as_str().unwrap()
                        );
                        ls_errors.push(s);
                    }
                }
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn duplicate_vertices(&self) -> Result<(), Vec<String>> {
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
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn materials(&self) -> Result<(), Vec<String>> {
        let mut max_index: usize = 0;
        let x = self.j["appearance"]["materials"].as_array();
        if x.is_some() {
            max_index = x.unwrap().len();
        }
        let mut ls_errors: Vec<String> = Vec::new();
        let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
        for theid in cos.keys() {
            //-- check geometry
            let x = self.j["CityObjects"][theid]["geometry"].as_array();
            if x.is_some() {
                let gs = x.unwrap();
                let mut gi = 0;
                for g in gs {
                    if g.get("material").is_none() {
                        continue;
                    }
                    if g["type"] == "MultiSurface" || g["type"] == "CompositeSurface" {
                        let bs = g["boundaries"].as_array().unwrap().len();
                        let gm = g["material"].as_object().unwrap();
                        for m_name in gm.keys() {
                            let gmv = g["material"][m_name]["values"].as_array();
                            if gmv.is_some() {
                                let x = gmv.unwrap();
                                if x.len() != bs {
                                    ls_errors.push(format!(
                                        "Material \"values\" not same dimension as \"boundaries\"; #{} / geom-#{} / material-\"{}\"", theid, gi, m_name
                                    ));
                                }
                                for each in x {
                                    if (each.as_u64().is_some())
                                        && (each.as_u64().unwrap() > (max_index - 1) as u64)
                                    {
                                        ls_errors.push(format!(
                                            "Reference in material \"values\" overflows (max={}); #{} and geom-#{} / material-\"{}\"",
                                            (max_index-1),theid, gi, m_name
                                        ));
                                    }
                                }
                            } else {
                                let ifvalue = g["material"][m_name]["value"].as_u64();
                                if ifvalue.is_some() {
                                    if ifvalue.unwrap() > (max_index - 1) as u64 {
                                        ls_errors.push(format!(
                                        "Material \"value\" overflow; #{} / geom-#{} / material-\"{}\"", theid, gi, m_name
                                        ));
                                    }
                                }
                            }
                        }
                    } else if g["type"] == "Solid" {
                        //-- length of the sem-surfaces == # of surfaces
                        let mut bs: Vec<usize> = Vec::new();
                        let shells = g["boundaries"].as_array().unwrap();
                        for shell in shells {
                            bs.push(shell.as_array().unwrap().len());
                        }
                        let gm = g["material"].as_object().unwrap();
                        for m_name in gm.keys() {
                            let mut vs: Vec<usize> = Vec::new();
                            let gmv = g["material"][m_name]["values"].as_array();
                            if gmv.is_some() {
                                let x = gmv.unwrap();
                                for each in x {
                                    let xa = each.as_array().unwrap();
                                    vs.push(xa.len());
                                    for each2 in xa {
                                        if (each2.as_u64().is_some())
                                            && (each2.as_u64().unwrap() > (max_index - 1) as u64)
                                        {
                                            ls_errors.push(format!(
                                                "Reference in material \"values\" overflows (max={}); #{} and geom-#{} / material-\"{}\"",
                                                (max_index-1),theid, gi, m_name
                                            ));
                                        }
                                    }
                                }
                            }
                            let ifvalue = g["material"][m_name]["value"].as_u64();
                            if ifvalue.is_some() {
                                if ifvalue.unwrap() > (max_index - 1) as u64 {
                                    ls_errors.push(format!(
                                    "Material \"value\" overflow; #{} / geom-#{} / material-\"{}\"", theid, gi, m_name
                                ));
                                }
                            } else {
                                if bs.iter().eq(vs.iter()) == false {
                                    ls_errors.push(format!(
                                    "Material \"values\" not same dimension as \"boundaries\"; #{} / geom-#{} / material-\"{}\"", theid, gi, m_name
                                ));
                                }
                            }
                        }
                    } else if g["type"] == "MultiSolid" || g["type"] == "CompositeSolid" {
                        //-- length of the sem-surfaces == # of surfaces
                        let mut bs: Vec<Vec<usize>> = Vec::new();
                        let solids = g["boundaries"].as_array().unwrap();
                        for solid in solids {
                            let asolid = solid.as_array().unwrap();
                            let mut tmp: Vec<usize> = Vec::new();
                            for surface in asolid {
                                tmp.push(surface.as_array().unwrap().len());
                            }
                            bs.push(tmp);
                        }
                        // println!("ms-bs: {:?}", bs);
                        let gm = g["material"].as_object().unwrap();
                        for m_name in gm.keys() {
                            let mut vs: Vec<Vec<usize>> = Vec::new();
                            let gmv = g["material"][m_name]["values"].as_array();
                            if gmv.is_some() {
                                let x = gmv.unwrap();
                                for a1 in x {
                                    let y = a1.as_array().unwrap();
                                    let mut vs2: Vec<usize> = Vec::new();
                                    for a2 in y {
                                        let xa = a2.as_array().unwrap();
                                        vs2.push(xa.len());
                                        for each2 in xa {
                                            if (each2.as_u64().is_some())
                                                && (each2.as_u64().unwrap()
                                                    > (max_index - 1) as u64)
                                            {
                                                ls_errors.push(format!(
                                                    "Reference in material \"values\" overflows (max={}); #{} and geom-#{} / material-\"{}\"",
                                                    (max_index-1),theid, gi, m_name
                                                ));
                                            }
                                        }
                                    }
                                    vs.push(vs2);
                                }
                            }
                            let ifvalue = g["material"][m_name]["value"].as_u64();
                            if ifvalue.is_some() {
                                if ifvalue.unwrap() > (max_index - 1) as u64 {
                                    ls_errors.push(format!(
                                    "Material \"value\" overflow; #{} / geom-#{} / material-\"{}\"", theid, gi, m_name
                                ));
                                }
                            } else {
                                if bs.iter().eq(vs.iter()) == false {
                                    ls_errors.push(format!(
                                    "Material \"values\" not same dimension as \"boundaries\"; #{} / geom-#{} / material-\"{}\"", theid, gi, m_name
                                ));
                                }
                            }
                        }
                    }
                    gi += 1;
                }
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn textures(&self) -> Result<(), Vec<String>> {
        let mut max_i_tex: usize = 0;
        let mut x = self.j["appearance"]["textures"].as_array();
        if x.is_some() {
            max_i_tex = x.unwrap().len();
        }
        let mut max_i_v: usize = 0;
        x = self.j["appearance"]["vertices-texture"].as_array();
        if x.is_some() {
            max_i_v = x.unwrap().len();
        }
        let mut ls_errors: Vec<String> = Vec::new();
        let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
        for theid in cos.keys() {
            //-- check geometry
            let x = self.j["CityObjects"][theid]["geometry"].as_array();
            if x.is_some() {
                let gs = x.unwrap();
                let mut gi = 0;
                for g in gs {
                    if g.get("texture").is_none() {
                        continue;
                    }
                    if g["type"] == "MultiSurface" || g["type"] == "CompositeSurface" {
                        let gs: GeomMSu = serde_json::from_value(g.clone()).unwrap();
                        let mut l: Vec<Vec<i64>> = Vec::new();
                        for x in gs.boundaries {
                            let mut l4: Vec<i64> = Vec::new();
                            for y in x {
                                l4.push(y.len() as i64);
                            }
                            l.push(l4);
                        }
                        let tex = g["texture"].as_object().unwrap();
                        for m_name in tex.keys() {
                            let ts: TextureMSu =
                                serde_json::from_value(g["texture"][m_name].clone()).unwrap();
                            let mut l2: Vec<Vec<i64>> = Vec::new();
                            for x in ts.values {
                                let mut l3: Vec<i64> = Vec::new();
                                for mut y in x {
                                    if y[0].is_none() {
                                        l3.push(-1);
                                    } else {
                                        l3.push(y.len() as i64 - 1);
                                    }
                                    if y.len() > 1 {
                                        if y[0].unwrap() > (max_i_tex - 1) {
                                            ls_errors.push(format!(
                                                    "/texture/values/ \"{}\" overflows for texture reference; #{} and geom-#{}",
                                                    y[0].unwrap(), theid, gi
                                                ));
                                        }
                                        y.remove(0);
                                        for each in y {
                                            if each.unwrap() > (max_i_v - 1) {
                                                ls_errors.push(format!(
                                                        "/texture/values/ \"{}\" overflows for texture-vertices (max={}); #{} and geom-#{}",
                                                        each.unwrap(), (max_i_v - 1), theid, gi
                                                    ));
                                            }
                                        }
                                    }
                                }
                                l2.push(l3);
                            }
                            if l != l2 {
                                for (i, _e) in l.iter().enumerate() {
                                    if l[i] != l2[i] && l2[i][0] != -1 {
                                        ls_errors.push(format!(
                                            "/texture/values/ not same structure as /boundaries; #{} and geom-#{} and surface-#{}", theid, gi, i
                                        ));
                                    }
                                }
                            }
                        }
                    } else if g["type"] == "Solid" {
                        let gs: GeomSol = serde_json::from_value(g.clone()).unwrap();
                        let mut l: Vec<Vec<i64>> = Vec::new();
                        for x in gs.boundaries {
                            for y in x {
                                let mut l4: Vec<i64> = Vec::new();
                                for z in y {
                                    l4.push(z.len() as i64);
                                }
                                l.push(l4);
                            }
                        }
                        let tex = g["texture"].as_object().unwrap();
                        for m_name in tex.keys() {
                            let ts: TextureSol =
                                serde_json::from_value(g["texture"][m_name].clone()).unwrap();
                            let mut l2: Vec<Vec<i64>> = Vec::new();
                            for x in ts.values {
                                for y in x {
                                    let mut l3: Vec<i64> = Vec::new();
                                    for mut z in y {
                                        if z[0].is_none() {
                                            l3.push(-1);
                                        } else {
                                            l3.push(z.len() as i64 - 1);
                                        }
                                        if z.len() > 1 {
                                            if z[0].unwrap() > (max_i_tex - 1) {
                                                ls_errors.push(format!(
                                                "/texture/values/ \"{}\" overflows for texture reference; #{} and geom-#{}",
                                                z[0].unwrap(), theid, gi
                                            ));
                                            }
                                            z.remove(0);
                                            for each in z {
                                                if each.unwrap() > (max_i_v - 1) {
                                                    ls_errors.push(format!(
                                                    "/texture/values/ \"{}\" overflows for texture-vertices (max={}); #{} and geom-#{}",
                                                    each.unwrap(), (max_i_v - 1), theid, gi
                                                ));
                                                }
                                            }
                                        }
                                    }
                                    l2.push(l3);
                                }
                            }
                            if l != l2 {
                                for (i, _e) in l.iter().enumerate() {
                                    if l[i] != l2[i] && l2[i][0] != -1 {
                                        ls_errors.push(format!(
                                            "/texture/values/ not same structure as /boundaries; #{} and geom-#{} and surface-#{}", theid, gi, i
                                        ));
                                    }
                                }
                            }
                        }
                    } else if g["type"] == "MultiSolid" || g["type"] == "CompositeSolid" {
                        let gs: GeomMSol = serde_json::from_value(g.clone()).unwrap();
                        let mut l: Vec<Vec<i64>> = Vec::new();
                        for x in gs.boundaries {
                            for y in x {
                                for z in y {
                                    let mut l4: Vec<i64> = Vec::new();
                                    for w in z {
                                        l4.push(w.len() as i64);
                                    }
                                    l.push(l4);
                                }
                            }
                        }
                        let tex = g["texture"].as_object().unwrap();
                        for m_name in tex.keys() {
                            let ts: TextureMSol =
                                serde_json::from_value(g["texture"][m_name].clone()).unwrap();
                            let mut l2: Vec<Vec<i64>> = Vec::new();
                            for x in ts.values {
                                for y in x {
                                    for z in y {
                                        let mut l3: Vec<i64> = Vec::new();
                                        for mut w in z {
                                            if w[0].is_none() {
                                                l3.push(-1);
                                            } else {
                                                l3.push(w.len() as i64 - 1);
                                            }
                                            if w.len() > 1 {
                                                if w[0].unwrap() > (max_i_tex - 1) {
                                                    ls_errors.push(format!(
                                                    "/texture/values/ \"{}\" overflows for texture reference; #{} and geom-#{}",
                                                    w[0].unwrap(), theid, gi
                                                ));
                                                }
                                                w.remove(0);
                                                for each in w {
                                                    if each.unwrap() > (max_i_v - 1) {
                                                        ls_errors.push(format!(
                                                        "/texture/values/ \"{}\" overflows for texture-vertices (max={}); #{} and geom-#{}",
                                                        each.unwrap(), (max_i_v - 1), theid, gi
                                                    ));
                                                    }
                                                }
                                            }
                                        }
                                        l2.push(l3);
                                    }
                                }
                            }
                            if l != l2 {
                                for (i, _e) in l.iter().enumerate() {
                                    if l[i] != l2[i] && l2[i][0] != -1 {
                                        ls_errors.push(format!(
                                            "/texture/values/ not same structure as /boundaries; #{} and geom-#{} and surface-#{}", theid, gi, i
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    gi += 1;
                }
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn wrong_vertex_index(&self) -> Result<(), Vec<String>> {
        let max_index: usize = self.j.get("vertices").unwrap().as_array().unwrap().len();
        let mut ls_errors: Vec<String> = Vec::new();
        let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
        for key in cos.keys() {
            //-- check geometry
            let x = self.j["CityObjects"][key]["geometry"].as_array();
            if x.is_some() {
                for g in x.unwrap() {
                    if g["type"] == "MultiPoint" {
                        let a: GeomMPo = serde_json::from_value(g.clone()).unwrap();
                        for each in a.boundaries {
                            if each >= max_index {
                                let s2 = format!("Vertices {} don't exist", each);
                                ls_errors.push(s2);
                            }
                        }
                    } else if g["type"] == "MultiLineString" {
                        let a: GeomMLS = serde_json::from_value(g.clone()).unwrap();
                        for l in a.boundaries {
                            for each in l {
                                if each >= max_index {
                                    let s2 = format!("Vertices {} don't exist", each);
                                    ls_errors.push(s2);
                                }
                            }
                        }
                    } else if g["type"] == "MultiSurface" || g["type"] == "CompositeSurface" {
                        let a: GeomMSu = serde_json::from_value(g.clone()).unwrap();
                        let re = above_max_index_msu(&a.boundaries, max_index);
                        if re.is_err() {
                            ls_errors.push(re.err().unwrap());
                        }
                    } else if g["type"] == "Solid" {
                        let a: GeomSol = serde_json::from_value(g.clone()).unwrap();
                        let re = above_max_index_sol(&a.boundaries, max_index);
                        if re.is_err() {
                            ls_errors.push(re.err().unwrap());
                        }
                    } else if g["type"] == "MultiSolid" || g["type"] == "CompositeSolid" {
                        let a: GeomMSol = serde_json::from_value(g.clone()).unwrap();
                        let re = above_max_index_msol(&a.boundaries, max_index);
                        if re.is_err() {
                            ls_errors.push(re.err().unwrap());
                        }
                    } else if g["type"] == "GeometryInstance" {
                        let a: GeomMPo = serde_json::from_value(g.clone()).unwrap();
                        for each in a.boundaries {
                            if each >= max_index {
                                let s2 = format!("Vertex {} doesn't exist (in #{})", each, key);
                                ls_errors.push(s2);
                            }
                        }
                    }
                }
            }
            //-- check address
            if self.j["CityObjects"][key]["type"] == "Building"
                || self.j["CityObjects"][key]["type"] == "BuildingPart"
                || self.j["CityObjects"][key]["type"] == "BuildingUnit"
                || self.j["CityObjects"][key]["type"] == "Bridge"
                || self.j["CityObjects"][key]["type"] == "BridgePart"
            {
                let x = self.j["CityObjects"][key]["address"].as_array();
                if x.is_some() {
                    for ad in x.unwrap() {
                        let t = ad.pointer("/location/boundaries");
                        if t.is_some() {
                            let i = t.unwrap().get(0).unwrap().as_u64().unwrap();
                            if (i as usize) >= max_index {
                                let s2 = format!("Vertices {} don't exist", i);
                                ls_errors.push(s2);
                            }
                        }
                    }
                }
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn unused_vertices(&self) -> Result<(), Vec<String>> {
        let mut ls_errors: Vec<String> = Vec::new();
        let mut uniques: HashSet<usize> = HashSet::new();
        let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
        for key in cos.keys() {
            //-- check geometry
            let x = self.j["CityObjects"][key]["geometry"].as_array();
            if x.is_some() {
                let gs = x.unwrap();
                for g in gs {
                    if g["type"] == "MultiPoint" {
                        let a: GeomMPo = serde_json::from_value(g.clone()).unwrap();
                        for each in a.boundaries {
                            uniques.insert(each);
                        }
                    } else if g["type"] == "MultiLineString" {
                        let a: GeomMLS = serde_json::from_value(g.clone()).unwrap();
                        for l in a.boundaries {
                            for each in l {
                                uniques.insert(each);
                            }
                        }
                    } else if g["type"] == "MultiSurface" || g["type"] == "CompositeSurface" {
                        let gv: GeomMSu = serde_json::from_value(g.clone()).unwrap();
                        collect_indices_msu(&gv.boundaries, &mut uniques);
                    } else if g["type"] == "Solid" {
                        let gv: GeomSol = serde_json::from_value(g.clone()).unwrap();
                        collect_indices_sol(&gv.boundaries, &mut uniques);
                    } else if g["type"] == "MultiSolid" || g["type"] == "CompositeSolid" {
                        let gv: GeomMSol = serde_json::from_value(g.clone()).unwrap();
                        collect_indices_msol(&gv.boundaries, &mut uniques);
                    } else if g["type"] == "GeometryInstance" {
                        let a: GeomMPo = serde_json::from_value(g.clone()).unwrap();
                        for each in a.boundaries {
                            uniques.insert(each);
                        }
                    }
                }
            }
            //-- check address
            if self.j["CityObjects"][key]["type"] == "Building"
                || self.j["CityObjects"][key]["type"] == "BuildingPart"
                || self.j["CityObjects"][key]["type"] == "BuildingUnit"
                || self.j["CityObjects"][key]["type"] == "Bridge"
                || self.j["CityObjects"][key]["type"] == "BridgePart"
            {
                let x = self.j["CityObjects"][key]["address"].as_array();
                if x.is_some() {
                    for ad in x.unwrap() {
                        let t = ad.pointer("/location/boundaries");
                        if t.is_some() {
                            let i = t.unwrap().get(0).unwrap().as_u64().unwrap();
                            uniques.insert(i as usize);
                        }
                    }
                }
            }
        }
        let noorphans = self.j["vertices"].as_array().unwrap().len() - uniques.len();
        if noorphans > 0 {
            if noorphans > 5 {
                ls_errors.push(format!("{} vertices are unused", noorphans));
            } else {
                let total = self.j["vertices"].as_array().unwrap().len();
                for each in 0..total {
                    if !uniques.contains(&each) {
                        ls_errors.push(format!("Vertex #{} is unused", each));
                    }
                }
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }

    fn semantics_arrays(&self) -> Result<(), Vec<String>> {
        let mut ls_errors: Vec<String> = Vec::new();
        let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
        for theid in cos.keys() {
            let x = self.j["CityObjects"][theid]["geometry"].as_array();
            if x.is_some() {
                let gs = x.unwrap();
                let mut gi = 0;
                for g in gs {
                    if g.get("semantics").is_none() {
                        continue;
                    }
                    if g["type"] == "MultiPoint"
                        || g["type"] == "MultiLineString"
                        || g["type"] == "MultiSurface"
                        || g["type"] == "CompositeSurface"
                    {
                        //-- length of the sem-surfaces == # of surfaces
                        if g["boundaries"].as_array().unwrap().len()
                            != g["semantics"]["values"].as_array().unwrap().len()
                        {
                            ls_errors.push(format!(
                                "Semantic \"values\" not same dimension as \"boundaries\"; #{} and geom-#{}", theid, gi
                            ));
                        }
                        //-- values in "values"
                        let a = g["semantics"]["surfaces"].as_array().unwrap().len();
                        for i in g["semantics"]["values"].as_array().unwrap() {
                            if i.is_null() {
                                continue;
                            }
                            if i.as_u64().unwrap() > (a - 1) as u64 {
                                ls_errors.push(format!(
                                    "Reference in semantic \"values\" overflows; #{} and geom-#{}",
                                    theid, gi
                                ));
                            }
                        }
                    }
                    if g["type"] == "Solid" {
                        //-- length of the sem-surfaces == # of surfaces
                        let mut bs: Vec<usize> = Vec::new();
                        let shells = g["boundaries"].as_array().unwrap();
                        for surface in shells {
                            bs.push(surface.as_array().unwrap().len());
                        }
                        // println!("bs: {:?}", bs);
                        let mut vs: Vec<usize> = Vec::new();
                        let tmp = g["semantics"]["values"].as_array().unwrap();
                        for each in tmp {
                            vs.push(each.as_array().unwrap().len());
                        }
                        // println!("vs: {:?}", vs);
                        // println!("eq: {:?}", bs.iter().eq(vs.iter()));
                        if bs.iter().eq(vs.iter()) == false {
                            ls_errors.push(format!(
                                "Semantic \"values\" not same dimension as \"boundaries\"; #{} and geom-#{}", theid, gi
                            ));
                        }
                        //-- values in "values"
                        let a = g["semantics"]["surfaces"].as_array().unwrap().len();
                        for i in g["semantics"]["values"].as_array().unwrap() {
                            let ai = i.as_array().unwrap();
                            for j in ai {
                                if j.is_null() {
                                    continue;
                                }
                                if j.as_u64().unwrap() > (a - 1) as u64 {
                                    ls_errors.push(format!(
                                        "Reference in semantic \"values\" overflows; #{} and geom-#{}",
                                        theid, gi
                                    ));
                                }
                            }
                        }
                    }
                    if g["type"] == "MultiSolid" || g["type"] == "CompositeSolid" {
                        //-- length of the sem-surfaces == # of surfaces
                        let mut bs: Vec<Vec<usize>> = Vec::new();
                        let solids = g["boundaries"].as_array().unwrap();
                        for solid in solids {
                            let asolid = solid.as_array().unwrap();
                            let mut tmp: Vec<usize> = Vec::new();
                            for surface in asolid {
                                tmp.push(surface.as_array().unwrap().len());
                            }
                            bs.push(tmp);
                        }
                        // println!("ms-bs: {:?}", bs);
                        let mut vs: Vec<Vec<usize>> = Vec::new();
                        let a = g["semantics"]["values"].as_array().unwrap();
                        for i in a {
                            let mut tmp: Vec<usize> = Vec::new();
                            let b = i.as_array().unwrap();
                            for j in b {
                                tmp.push(j.as_array().unwrap().len());
                            }
                            vs.push(tmp);
                        }
                        // println!("ms-vs: {:?}", vs);
                        // println!("eq: {:?}", bs.iter().eq(vs.iter()));
                        if bs.iter().eq(vs.iter()) == false {
                            ls_errors.push(format!(
                                "Semantic \"values\" not same dimension as \"boundaries\"; #{} and geom-#{}", theid, gi
                            ));
                        }
                        //-- values in "values"
                        let a = g["semantics"]["surfaces"].as_array().unwrap().len();
                        for i in g["semantics"]["values"].as_array().unwrap() {
                            let ai = i.as_array().unwrap();
                            for j in ai {
                                let aj = j.as_array().unwrap();
                                for k in aj {
                                    if k.is_null() {
                                        continue;
                                    }
                                    if k.as_u64().unwrap() > (a - 1) as u64 {
                                        ls_errors.push(format!(
                                        "Reference in semantic \"values\" overflows; #{} and geom-#{}",
                                        theid, gi
                                    ));
                                    }
                                }
                            }
                        }
                    }
                    gi += 1;
                }
            }
        }
        if ls_errors.is_empty() {
            Ok(())
        } else {
            Err(ls_errors)
        }
    }
}

fn collect_indices_msu(a: &Vec<Vec<Vec<usize>>>, uniques: &mut HashSet<usize>) {
    for x in a {
        for y in x {
            for z in y {
                uniques.insert(*z);
            }
        }
    }
}

fn collect_indices_sol(a: &Vec<Vec<Vec<Vec<usize>>>>, uniques: &mut HashSet<usize>) {
    for x in a {
        for y in x {
            for z in y {
                for w in z {
                    uniques.insert(*w);
                }
            }
        }
    }
}

fn collect_indices_msol(a: &Vec<Vec<Vec<Vec<Vec<usize>>>>>, uniques: &mut HashSet<usize>) {
    for x in a {
        for y in x {
            for z in y {
                for w in z {
                    for q in w {
                        uniques.insert(*q);
                    }
                }
            }
        }
    }
}

fn above_max_index_msu(a: &Vec<Vec<Vec<usize>>>, max_index: usize) -> Result<(), String> {
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
        Ok(())
    } else {
        let mut s: String = "".to_string();
        for each in r {
            s += "#";
            s += &each.to_string();
            s += "/";
        }
        let s2 = format!("Vertices {} don't exist", s);
        Err(s2)
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
        Ok(())
    } else {
        let mut s: String = "".to_string();
        for each in r {
            s += "#";
            s += &each.to_string();
            s += "/";
        }
        let s2 = format!("Vertices {} don't exist", s);
        Err(s2)
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
