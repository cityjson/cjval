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
                "id-1": {
                  "type": "Building",
                  "geometry": [
                  {
                    "type": "MultiSurface",
                    "lod": "2",
                    "boundaries": [
                      [[0, 3, 2, 1]], [[4, 5, 6, 7]], [[0, 1, 5, 4]], [[0, 2, 3, 4]], [[3, 4, 2, 3]]
                    ],
                    "semantics": {
                      "surfaces" : [
                        {
                          "type": "WallSurface",
                          "slope": 33.4,
                          "children": [2]
                        }, 
                        {
                          "type": "RoofSurface",
                          "slope": 66.6
                        },
                        {
                          "type": "+PatioDoor",
                          "parent": 0,
                          "colour": "blue"
                        }
                      ],
                      "values": [0, 0, null, 1, 1]
                    }
                  }
                  ],
                  "attributes": {
                    "function2": "something"
                  }
                }
            },
            "vertices": [
                [0, 0, 0],
                [1000, 0, 0],
                [1000, 1000, 0],
                [1000, 88, 0],
                [1000, 7, 0],
                [1000, 6, 33],
                [1000, 4, 0],
                [1000, 3, 43]
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
fn semantics_arrays_valid() {
    let j = get_data();
    let v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.semantics_arrays();
    assert!(re.is_empty());
}

#[test]
fn semantics_arrays_diff_dimensions() {
    let mut j = get_data();
    j["CityObjects"]["id-1"]["geometry"][0]["semantics"]["values"]
        .as_array_mut()
        .unwrap()
        .push(json!(1));
    // println!("=====>{:?}", &j);
    let v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.semantics_arrays();
    assert!(!re.is_empty());
}

#[test]
fn semantics_arrays_unused() {
    let mut j = get_data();
    j["CityObjects"]["id-1"]["geometry"][0]["semantics"]["values"]
        .as_array_mut()
        .unwrap()
        .pop();
    j["CityObjects"]["id-1"]["geometry"][0]["semantics"]["values"]
        .as_array_mut()
        .unwrap()
        .push(json!(77));
    let v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.semantics_arrays();
    assert!(!re.is_empty());
}
