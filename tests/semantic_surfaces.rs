use crate::cjval::CJValidator;
use cjval;

use serde_json::Value;

fn get_data_v11() -> Value {
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

fn get_data_v20() -> Value {
    let j_mininal = r#"
        {
            "type": "CityJSON",
            "version": "2.0",
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
fn sem_surfaces_v11() {
    let j = get_data_v11();
    let v: CJValidator = CJValidator::from_str(&j.to_string());
    let re = v.validate();
    assert!(re["extensions"].is_valid());
}

#[test]
fn sem_surfaces_v20() {
    let j = get_data_v20();
    let v: CJValidator = CJValidator::from_str(&j.to_string());
    let re = v.validate();
    assert!(!re["extensions"].is_valid());
}
