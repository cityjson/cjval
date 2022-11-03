use crate::cjval::CJValidator;
use cjval;

use serde_json::Value;

fn get_data() -> Value {
    let j_mininal = r#"
        {
            "type": "CityJSON",
            "version": "1.1",
            "CityObjects":
            {
                "LondonTower": { "type": "Building" }
            },
            "vertices": [
                [0, 0, 0],
                [1000, 0, 0],
                [1000, 1000, 0],
                [1000, 0, 0]
            ],
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
fn invalid_integer() {
    let j = get_data();
    let v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.duplicate_vertices();
    assert!(re.is_err());
}

#[test]
fn valid_integer() {
    let mut j = get_data();
    j["vertices"].as_array_mut().unwrap().pop().unwrap();
    let v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.duplicate_vertices();
    assert!(re.is_ok());
}
