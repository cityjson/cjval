use crate::cjval::CJValidator;
use cjval;
use serde_json::json;
use serde_json::Value;

fn get_first_line() -> Value {
    let j_1 = r#"
        {
          "type": "CityJSON",
          "version": "2.0",
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
              "url": "https://cityjson.org/extensions/download/v20/generic.ext.json",
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
                      [ [17, 6, 5, 4 ] ]
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
                  "lod": "2.2",
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
    let mut v: CJValidator = CJValidator::from_str(&j.to_string());
    let mut re = v.validate();
    assert!(re["schema"].is_valid());

    j = get_second_line();
    let _ = v.from_str_cjfeature(&j.to_string());
    re = v.validate();
    assert!(re["schema"].is_valid());
}

#[test]
fn cjfeature_invalid() {
    let mut j = get_first_line();
    let mut v: CJValidator = CJValidator::from_str(&j.to_string());
    let mut re = v.validate();
    assert!(re["schema"].is_valid());

    j = get_second_line();
    let _ = v.from_str_cjfeature(&j.to_string());
    re = v.validate();
    assert!(re["schema"].is_valid());

    *j.pointer_mut("/CityObjects/id-1/geometry/0/lod").unwrap() = json!("1.5");
    let _ = v.from_str_cjfeature(&j.to_string());
    re = v.validate();
    assert!(!re["schema"].is_valid());
}

#[test]
fn cjfeature_extension() {
    let mut j = get_first_line();
    let mut v: CJValidator = CJValidator::from_str(&j.to_string());
    let s = std::fs::read_to_string("schemas/extensions/20/generic.ext.json").unwrap();
    let _ = v.add_one_extension_from_str(&s);
    let mut re = v.validate();
    assert!(re["schema"].is_valid());

    j = get_third_line();
    let _ = v.from_str_cjfeature(&j.to_string());
    re = v.validate();
    assert!(re["schema"].is_valid());
    assert!(re["extensions"].is_valid());
}
