use crate::cjval::CJValidator;
use cjval;
use serde_json::json;
use serde_json::Value;

fn get_data() -> Value {
    let j_mininal = r#"
        {
            "type": "CityJSON",
            "version": "1.1",
            "CityObjects":
            {
                "LondonTower":
                {
                    "type": "Building",
                    "children":
                    [
                        "oneroom"
                    ]
                },
                "oneroom":
                {
                    "type": "BuildingRoom",
                    "parents":
                    [
                        "LondonTower"
                    ]
                }
            },
            "vertices": [],
            "transform":
            {
                "scale":
                [
                    0.001,
                    0.001,
                    0.001
                ],
                "translate":
                [
                    0.0,
                    0.0,
                    0.0
                ]
            }
        }
        "#;
    let v: Value = serde_json::from_str(&j_mininal).unwrap();
    v
}

#[test]
fn valid() {
    let j = get_data();
    let v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.validate_schema();
    assert!(re.is_ok());
}

#[test]
fn no_child() {
    let mut j = get_data();
    j["CityObjects"]["LondonTower"]["children"]
        .as_array_mut()
        .unwrap()
        .push(json!("hugo"));
    // println!("=====>{:?}", ar);
    let v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.parent_children_consistency();
    assert!(re.is_err());
}

#[test]
fn no_parent() {
    let mut j = get_data();
    j["CityObjects"]["oneroom"]["parents"]
        .as_array_mut()
        .unwrap()
        .push(json!("hugo"));
    // println!("=====>{:?}", ar);
    let v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.parent_children_consistency();
    assert!(re.is_err());
}
