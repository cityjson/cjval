use crate::cjval::CJValidator;
use cjval;
use serde_json::json;
use serde_json::Value;

fn get_first_line() -> Value {
    let j_1 = r#"
        {
          "type": "CityJSON",
          "version": "1.1",
          "CityObjects": {},
          "vertices": [],
          "transform": {
            "scale": [
              0.001,
              0.001,
              0.001
            ],
            "translate": [
              -1.0,
              -1.0,
              0.0
            ]
          },
          "metadata": {
            "geographicalExtent": [
              -1.0,
              -1.0,
              0.0,
              1.0,
              1.0,
              1.0
            ]
          },
          "extensions": {
            "Generic": {
              "url": "https://cityjson.org/extensions/download/generic.ext.json",
              "version": "1.0"
            }
          }
        }
        "#;
    let v: Value = serde_json::from_str(&j_1).unwrap();
    v
}

fn get_second_line() -> Value {
    let j_1 = r#"
        {
          "type": "CityJSONFeature",
          "CityObjects": {
            "id-1": {
              "geometry": [
                {
                  "boundaries": [
                    [
                      [ [0, 1, 2, 3 ] ], 
                      [ [4, 5, 0, 3 ] ], 
                      [ [5, 6, 1, 0 ] ], 
                      [ [6, 7, 2, 1 ] ], 
                      [ [3, 2, 7, 4 ] ], 
                      [ [7, 6, 5, 4 ] ] 
                    ]
                  ],
                  "lod": "1.2",
                  "type": "Solid"
                }
              ],
              "attributes": {
                "function": "something"
              },
              "type": "Building"
            }
          },
          "vertices": [
            [2000, 1000, 1000 ],
            [1000, 2000, 1000 ],
            [0, 1000, 1000 ],
            [1000, 0, 1000 ],
            [1000, 0, 0 ],
            [2000, 1000, 0 ],
            [1000, 2000, 0 ], 
            [0, 1000, 0 ] 
          ],
          "id": "id-1"
        }
        "#;
    let v: Value = serde_json::from_str(&j_1).unwrap();
    v
}

fn get_third_line() -> Value {
    let j_1 = r#"
        {
          "type": "CityJSONFeature",
          "CityObjects": {
            "id-1": {
              "geometry": [
                {
                  "boundaries": [
                    [
                      [ [0, 1, 2, 3 ] ], 
                      [ [4, 5, 0, 3 ] ], 
                      [ [5, 6, 1, 0 ] ], 
                      [ [6, 7, 2, 1 ] ], 
                      [ [3, 2, 7, 4 ] ], 
                      [ [7, 6, 5, 4 ] ] 
                    ]
                  ],
                  "lod": "1",
                  "type": "Solid"
                }
              ],
              "attributes": {
                "function": "something"
              },
              "type": "+GenericCityObject"
            }
          },
          "vertices": [
            [2000, 1000, 1000 ],
            [1000, 2000, 1000 ],
            [0, 1000, 1000 ],
            [1000, 0, 1000 ],
            [1000, 0, 0 ],
            [2000, 1000, 0 ],
            [1000, 2000, 0 ], 
            [0, 1000, 0 ] 
          ],
          "id": "id-1"
        }
        "#;
    let v: Value = serde_json::from_str(&j_1).unwrap();
    v
}

#[test]
fn cjfeature_valid() {
    let mut j = get_first_line();
    let mut v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let mut re = v.validate_schema();
    assert!(re.is_empty());

    j = get_second_line();
    v = CJValidator::from_str(&j.to_string()).unwrap();
    re = v.validate_schema();
    assert!(re.is_empty());
}

#[test]
fn cjfeature_extension() {
    let j = get_third_line();
    let v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.validate_schema();
    assert!(re.is_empty());
}
