use crate::cjval::CJValidator;
use cjval;
use serde_json::json;

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
                            [[0, 3, 2, 1]], [[4, 5, 6, 8]], [[0, 1, 5, 4]]
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
                          "boundaries": [2, 0, 8]
                        }
                    ]
                },
                "deux": { 
                    "type": "CityFurniture",
                    "geometry": [
                        {
                          "type": "MultiLineString",
                          "lod": "1",
                          "boundaries": [ [2, 0, 8], [3, 8] ]
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
fn wrong_vertex_index_1() {
    let j = get_data1();
    let v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.wrong_vertex_index();
    assert!(!re.is_empty());
}

#[test]
fn wrong_vertex_index_2() {
    let mut j = get_data2();
    let mut v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.wrong_vertex_index();
    assert!(re.is_empty());

    j["CityObjects"]["un"]["geometry"][0]["boundaries"]
        .as_array_mut()
        .unwrap()
        .push(json!(77));
    v = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.wrong_vertex_index();
    assert!(!re.is_empty());
}
