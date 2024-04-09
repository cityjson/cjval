use crate::cjval::CJValidator;
use cjval;
use serde_json::json;
use serde_json::Value;

fn get_minimal_11() -> Value {
    let j_mininal = r#"
        {
          "type": "CityJSON",
          "version": "1.1",
          "transform": {
            "scale": [0.0, 0.0, 0.0],
            "translate": [1.0, 1.0, 1.0]
          },
          "extensions":
          {
            "Generic":
            {
              "url": "https://www.cityjson.org/extensions/download/generic.ext.json",
              "version": "1.0"
            }
          },
          "CityObjects": {
            "un": {
              "type": "+GenericCityObject"
            }
          },
          "vertices": []
        }
        "#;
    let v: Value = serde_json::from_str(&j_mininal).unwrap();
    v
}

fn get_minimal_20() -> Value {
    let j_mininal = r#"
        {
          "type": "CityJSON",
          "version": "2.0",
          "transform": {
            "scale": [0.0, 0.0, 0.0],
            "translate": [1.0, 1.0, 1.0]
          },
          "extensions":
          {
            "Generic":
            {
              "url": "https://www.cityjson.org/extensions/download/v20/generic.ext.json",
              "version": "1.0"
            }
          },
          "CityObjects": {
            "un": {
              "type": "+GenericCityObject"
            }
          },
          "vertices": []
        }
        "#;
    let v: Value = serde_json::from_str(&j_mininal).unwrap();
    v
}

fn get_potato_20() -> Value {
    let j_mininal = r#"
        {
          "type": "CityJSON",
          "version": "2.0",
          "transform": {
            "scale": [0.0, 0.0, 0.0],
            "translate": [1.0, 1.0, 1.0]
          },
          "CityObjects": {
            "un": {
              "type": "+Potato",
              "geometry": [
                {
                  "boundaries":
                    [[ [[0, 1, 2, 3]], [[4, 5, 0, 3]], [[5, 6, 1, 0]], [[6, 7, 2, 1]], [[3, 2, 7, 4]], [[7, 6, 5, 4]] ]],
                  "lod": "1.3",
                  "type": "Solid"
                }
              ]
            }
          },
          "vertices": [[2000, 1000, 1000 ], [1000, 2000, 1000 ], [0, 1000, 1000 ], [1000, 0, 1000 ], [1000, 0, 0 ], [2000, 1000, 0 ], [1000, 2000, 0 ], [0, 1000, 0 ] ]
        }
        "#;
    let v: Value = serde_json::from_str(&j_mininal).unwrap();
    v
}

#[test]
fn extension_generic_11() {
    let mut j = get_minimal_11();
    let mut v: CJValidator = CJValidator::from_str(&j.to_string());
    let mut re = v.validate();
    assert!(re["schema"].is_valid());
    assert!(!re["extensions"].is_valid());

    let s = std::fs::read_to_string("schemas/extensions/11/generic.ext.json").unwrap();
    let _ = v.add_one_extension_from_str(&s);
    re = v.validate();
    assert!(re["extensions"].is_valid());

    *j.pointer_mut("/CityObjects/un/type").unwrap() = json!("GenericCityObject");
    v = CJValidator::from_str(&j.to_string());
    re = v.validate();
    assert!(!re["schema"].is_valid());

    *j.pointer_mut("/CityObjects/un/type").unwrap() = json!("+GenericCityObject2");
    v = CJValidator::from_str(&j.to_string());
    re = v.validate();
    assert!(re["schema"].is_valid());
    assert!(!re["extensions"].is_valid());
}

#[test]
fn extension_generic_20() {
    let mut j = get_minimal_20();
    let mut v: CJValidator = CJValidator::from_str(&j.to_string());
    let mut re = v.validate();
    assert!(re["schema"].is_valid());
    assert!(!re["extensions"].is_valid());

    let s = std::fs::read_to_string("schemas/extensions/20/generic.ext.json").unwrap();
    let _ = v.add_one_extension_from_str(&s);
    re = v.validate();
    assert!(re["extensions"].is_valid());

    *j.pointer_mut("/CityObjects/un/type").unwrap() = json!("GenericCityObject");
    v = CJValidator::from_str(&j.to_string());
    re = v.validate();
    assert!(re["schema"].is_valid());

    *j.pointer_mut("/CityObjects/un/type").unwrap() = json!("+GenericCityObject2");
    v = CJValidator::from_str(&j.to_string());
    re = v.validate();
    assert!(re["schema"].is_valid());
    assert!(!re["extensions"].is_valid());
}

#[test]
fn extension_noise() {
    let sdata = std::fs::read_to_string("data/noise1.city.json").unwrap();
    let mut v: CJValidator = CJValidator::from_str(&sdata);
    let re = v.validate();
    assert!(re["schema"].is_valid());

    let sschema = std::fs::read_to_string("schemas/extensions/11/noise.ext.json").unwrap();
    let _ = v.add_one_extension_from_str(&sschema);
    let rev = v.validate();
    assert!(rev["extensions"].is_valid());
}

#[test]
fn extension_20() {
    let mut j = get_potato_20();
    let mut v: CJValidator = CJValidator::from_str(&j.to_string());
    let mut re = v.validate();
    assert!(re["schema"].is_valid());
    assert!(!re["extensions"].is_valid());

    let s = std::fs::read_to_string("schemas/extensions/20/potato.ext.json").unwrap();
    let _ = v.add_one_extension_from_str(&s);
    re = v.validate();
    assert!(re["schema"].is_valid());
    assert!(re["extensions"].is_valid());

    *j.pointer_mut("/CityObjects/un/geometry/0/lod").unwrap() = json!("1.5");
    // println!("{:?}", j);
    v = CJValidator::from_str(&j.to_string());
    re = v.validate();
    assert!(re["schema"].is_valid());
    assert!(!re["extensions"].is_valid());
}
