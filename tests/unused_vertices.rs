use crate::cjval::CJValidator;
use cjval;

use serde_json::Value;

fn get_data1() -> Value {
    let j_mininal = r#"
        {
            "type": "CityJSON",
            "version": "1.1",
            "CityObjects":
            {
                "LondonTower": { 
                    "type": "Building",
                    "geometry": [
                        {
                          "type": "MultiSurface",
                          "lod": "2",
                          "boundaries": [
                            [[0, 3, 2, 1]], [[4, 5, 6, 7]], [[0, 1, 5, 4]]
                          ]
                        }
                    ]
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
                [1000, 3, 43],
                [100, 20, 45]
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

fn get_data2() -> Value {
    let j_mininal = r#"
        {
            "type": "CityJSON",
            "version": "1.1",
            "CityObjects":
            {
                "un": { 
                    "type": "CityFurniture",
                    "geometry": [
                        {
                          "type": "MultiPoint",
                          "lod": "1",
                          "boundaries": [0, 1, 2]
                        }
                    ]
                },
                "deux": { 
                    "type": "CityFurniture",
                    "geometry": [
                        {
                          "type": "MultiLineString",
                          "lod": "1",
                          "boundaries": [ [3, 4, 5], [6, 7] ]
                        }
                    ]
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
                [1000, 3, 43],
                [100, 20, 45]
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
fn unused_vertex_1() {
    let j = get_data1();
    let v: CJValidator = CJValidator::from_str(&j.to_string());
    let re = v.validate();
    assert!(!re["unused_vertices"].is_valid());
}

#[test]
fn unused_vertex_2() {
    let j = get_data2();
    let v: CJValidator = CJValidator::from_str(&j.to_string());
    let re = v.validate();
    assert!(!re["unused_vertices"].is_valid());
}
