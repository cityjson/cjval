use crate::cjval::CJValidator;
use cjval;

use serde_json::json;
use serde_json::Value;

fn get_data1() -> Value {
    let j_mininal = r#"
        {
            "type": "CityJSON",
            "version": "2.0",
            "CityObjects":
            {
                "LondonTower": { 
                    "type": "Building",
                    "attributes": {
                      "size": 23232.1,
                      "+building-type": "residential",
                      "function": "something"
                    },
                    "geometry": []
                }
            },
            "vertices": [],
            "transform":
            {
                "scale": [0.001, 0.001, 0.001],
                "translate": [ 0.0, 0.0, 0.0]
            }
        }
        "#;
    let v: Value = serde_json::from_str(&j_mininal).unwrap();
    v
}

#[test]
fn extension_att_no_schema() {
    let j = get_data1();
    let v: CJValidator = CJValidator::from_str(&j.to_string());
    let re = v.validate();
    assert!(!re["extensions"].is_valid());
}

#[test]
fn extension_att_w_schema() {
    let j = get_data1();
    let mut v: CJValidator = CJValidator::from_str(&j.to_string());
    let s = std::fs::read_to_string("schemas/extensions/20/simpleattribute.ext.json").unwrap();
    let _ = v.add_one_extension_from_str(&s);
    let re = v.validate();
    assert!(re["extensions"].is_valid());
}

#[test]
fn extension_att_w_schema_2() {
    let mut j = get_data1();
    *j.pointer_mut("/CityObjects/LondonTower/attributes/+building-type")
        .unwrap() = json!("commercial");
    let mut v: CJValidator = CJValidator::from_str(&j.to_string());
    let s = std::fs::read_to_string("schemas/extensions/20/simpleattribute.ext.json").unwrap();
    let _ = v.add_one_extension_from_str(&s);
    let re = v.validate();
    assert!(re["extensions"].is_valid());
}
