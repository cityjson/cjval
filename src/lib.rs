use anyhow::{anyhow, Result};
use jsonschema::{Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;

// #-- ERRORS
//  # validate_schema
//  # validate_extensions
//  # parent_children_consistency
//  # wrong_vertex_index
//  # semantics_arrays
//
//
// #-- WARNINGS
//  # extra_root_properties
//  # duplicate_vertices
//  # unused_vertices

static EXTENSION_FIXED_NAMES: [&str; 6] = [
    "type",
    "name",
    "uri",
    "version",
    "versionCityJSON",
    "description",
    // "extraAttributes",
    // "extraCityObjects",
    // "extraRootProperties",
];

pub static CITYJSON_VERSIONS: [&str; 2] = ["1.0.3", "1.1.0"];

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
#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Doc {
    #[serde(with = "::serde_with::rust::maps_duplicate_key_is_error")]
    CityObjects: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct CJValidator {
    j: Value,
    jexts: HashMap<String, Value>,
    duplicate_keys: bool,
    version: i32,
}

impl CJValidator {
    pub fn from_str(str_dataset: &str) -> Result<Self, String> {
        let l: HashMap<String, Value> = HashMap::new();
        let mut v = CJValidator {
            j: json!(null),
            jexts: l,
            duplicate_keys: false,
            version: 0,
        };
        //-- parse the dataset and convert to JSON
        let re: Result<Value, _> = serde_json::from_str(&str_dataset);
        if re.is_err() {
            // println!("errors: {:?}", re.as_ref().err().unwrap());
            return Err(re.err().unwrap().to_string());
        }
        let j: Value = re.unwrap();
        v.j = j;
        //-- check cityjson version
        if v.j["version"] == "1.1" {
            v.version = 11;
        } else if v.j["version"] == "1.0" {
            v.version = 10;
        }
        //-- check for duplicate keys in CO object
        let re: Result<Doc, _> = serde_json::from_str(&str_dataset);
        if re.is_err() {
            // println!("{:?}", re.err());
            v.duplicate_keys = true;
        }
        Ok(v)
    }

    pub fn add_one_extension_from_str(
        &mut self,
        ext_schema_name: &str,
        ext_schema_str: &str,
    ) -> Result<()> {
        let re: Result<Value, _> = serde_json::from_str(ext_schema_str);
        if re.is_err() {
            return Err(anyhow!(re.err().unwrap().to_string()));
        }
        self.jexts.insert(ext_schema_name.to_string(), re.unwrap());
        Ok(())
    }

    /// Does the CityJSON contain Extension(s)?
    pub fn has_extensions(&self) -> Option<Vec<String>> {
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

    pub fn get_extensions(&self) -> &HashMap<String, Value> {
        &self.jexts
    }

    pub fn get_input_cityjson_version(&self) -> i32 {
        self.version
    }

    pub fn validate_schema(&self) -> Vec<String> {
        if self.j.is_null() {
            return vec!["Not a valid JSON file".to_string()];
        }
        //-- which cityjson version
        if self.version == 0 {
            let s: String = format!(
                "CityJSON version {} not supported [only \"1.0\" and \"1.1\"]",
                self.j["version"]
            );
            return vec![s];
        }
        //-- fetch the correct schema
        let mut schema_str = include_str!("../schemas/10/cityjson.min.schema.json");
        if self.version == 11 {
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

    fn validate_ext_extracityobjects(&self, jext: &Value) -> Vec<String> {
        let mut ls_errors: Vec<String> = Vec::new();
        //-- 1. build the schema file from the Extension file
        let v = jext.get("extraCityObjects").unwrap().as_object().unwrap();
        let jexto = jext.as_object().unwrap();
        for eco in v.keys() {
            // println!("==>{:?}", eco);
            let mut schema = jext["extraCityObjects"][eco].clone();
            schema["$schema"] = json!("http://json-schema.org/draft-07/schema#");
            schema["$id"] = json!("https://www.cityjson.org/schemas/1.1.0/tmp.json");
            for each in jexto.keys() {
                let ss = each.as_str();
                if EXTENSION_FIXED_NAMES.contains(&ss) == false {
                    schema[ss] = jext[ss].clone();
                }
            }
            // println!("=>{}", serde_json::to_string(&schema).unwrap());
            let compiled = self.get_compiled_schema_extension(&schema);
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
        ls_errors
    }

    fn validate_ext_extrarootproperties(&self, jext: &Value) -> Vec<String> {
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
            schema["$id"] = json!("https://www.cityjson.org/schemas/1.1.0/tmp.json");
            for each in jexto.keys() {
                let ss = each.as_str();
                if EXTENSION_FIXED_NAMES.contains(&ss) == false {
                    schema[ss] = jext[ss].clone();
                }
            }
            let compiled = self.get_compiled_schema_extension(&schema);

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
        ls_errors
    }

    fn validate_ext_extraattributes(&self, jext: &Value) -> Vec<String> {
        let mut ls_errors: Vec<String> = Vec::new();
        //-- 1. build the schema file from the Extension file
        let v = jext.get("extraAttributes").unwrap().as_object().unwrap();
        let jexto = jext.as_object().unwrap();
        for cotype in v.keys() {
            //-- for each CityObject type
            for eatt in jext["extraAttributes"][cotype].as_object().unwrap().keys() {
                let mut schema = jext["extraAttributes"][cotype][eatt.as_str()].clone();
                schema["$schema"] = json!("http://json-schema.org/draft-07/schema#");
                schema["$id"] = json!("https://www.cityjson.org/schemas/1.1.0/tmp.json");
                for each in jexto.keys() {
                    let ss = each.as_str();
                    if EXTENSION_FIXED_NAMES.contains(&ss) == false {
                        schema[ss] = jext[ss].clone();
                    }
                }
                let compiled = self.get_compiled_schema_extension(&schema);
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
        ls_errors
    }

    fn get_compiled_schema_extension(&self, schema: &Value) -> JSONSchema {
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
        return compiled;
    }

    pub fn validate_extensions(&self) -> Vec<String> {
        let mut ls_errors: Vec<String> = Vec::new();
        for (_k, ext) in &self.jexts {
            //-- 1. extraCityObjects
            ls_errors.append(&mut self.validate_ext_extracityobjects(&ext));
            //-- 2. extraRootProperties
            ls_errors.append(&mut self.validate_ext_extrarootproperties(&ext));
            //-- 3. extraAttributes
            ls_errors.append(&mut self.validate_ext_extraattributes(&ext));
        }
        //-- 4. check if there are CityObjects that do not have a schema
        ls_errors.append(&mut self.validate_ext_co_without_schema());
        //-- 5. check if there are extra root properties that do not have a schema
        ls_errors.append(&mut self.validate_ext_rootproperty_without_schema());
        //TODO 6 for the extra attributes w/o schemas
        ls_errors.append(&mut self.validate_ext_attribute_without_schema());
        ls_errors
    }

    fn validate_ext_attribute_without_schema(&self) -> Vec<String> {
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
            for (_theid, jext) in &self.jexts {
                let s = format!("/extraAttributes/{}", each);
                let re = jext.pointer(s.as_str());
                if re.is_none() {
                    let s: String = format!("Attribute '{}' doesn't have a schema", each);
                    ls_errors.push(s);
                }
            }
        }
        ls_errors
    }

    fn validate_ext_co_without_schema(&self) -> Vec<String> {
        let mut ls_errors: Vec<String> = Vec::new();
        let mut newcos: Vec<String> = Vec::new();
        for (_k, jext) in &self.jexts {
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
        ls_errors
    }

    fn validate_ext_rootproperty_without_schema(&self) -> Vec<String> {
        let mut ls_errors: Vec<String> = Vec::new();
        let mut newrps: Vec<String> = Vec::new();
        for (_k, jext) in &self.jexts {
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
        ls_errors
    }

    pub fn extra_root_properties(&self) -> Vec<String> {
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

        ls_warnings
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
        }
        ls_errors
    }

    pub fn unused_vertices(&self) -> Vec<String> {
        let mut ls_errors: Vec<String> = Vec::new();
        let mut uniques: HashSet<usize> = HashSet::new();
        let cos = self.j.get("CityObjects").unwrap().as_object().unwrap();
        for theid in cos.keys() {
            let x = self.j["CityObjects"][theid]["geometry"].as_array();
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
        ls_errors
    }

    pub fn semantics_arrays(&self) -> Vec<String> {
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
        ls_errors
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
