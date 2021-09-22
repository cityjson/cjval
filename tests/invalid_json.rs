use crate::cjval::CJValidator;
use cjval;
use serde_json::json;
use serde_json::Value;

fn get_minimal() -> Value {
    let j_mininal = r#"
        {
          "type": "CityJSON",
          "version": "1.1",
          "transform": {
            "scale": [0.0, 0.0, 0.0],
            "translate": [1.0, 1.0, 1.0]
          },
          "CityObjects": {},
          "vertices": []
        }
        "#;
    let v: Value = serde_json::from_str(&j_mininal).unwrap();
    v
}

#[test]
fn minimal() {
    let j = get_minimal();
    let v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.validate_schema();
    assert!(re.is_empty());

    let mut j2 = j.clone();
    j2.as_object_mut().unwrap().remove("vertices").unwrap();
    let v2: CJValidator = CJValidator::from_str(&j2.to_string()).unwrap();
    let re = v2.validate_schema();
    assert!(!re.is_empty());
}

#[test]
fn version() {
    let j_mininal = r#"
        {
          "type": "Potato",
          "version": "1.1",
          "transform": {
            "scale": [0.0, 0.0, 0.0],
            "translate": [1.0, 1.0, 1.0]
          },
          "CityObjects": {},
          "vertices": []
        }
        "#;
    let mut j: Value = serde_json::from_str(&j_mininal).unwrap();
    *j.get_mut("version").unwrap() = json!("1.0");
    let mut v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let mut re = v.validate_schema();
    assert!(!re.is_empty());

    j.as_object_mut().unwrap().remove("version");
    v = CJValidator::from_str(&j.to_string()).unwrap();
    re = v.validate_schema();
    assert!(!re.is_empty());
}

#[test]
fn non_cityjson() {
    let j_mininal = r#"
        {
          "type": "Potato",
          "version": "1.1",
          "transform": {
            "scale": [0.0, 0.0, 0.0],
            "translate": [1.0, 1.0, 1.0]
          },
          "CityObjects": {},
          "vertices": []
        }
        "#;
    let mut j: Value = serde_json::from_str(&j_mininal).unwrap();
    let mut v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let mut re = v.validate_schema();
    assert!(!re.is_empty());

    // *j.get_mut("type").unwrap() = json!("CityJSON");
    // v = CJValidator::from_str(&j.to_string());
    // re = v.validate_schema();
    // assert!(re.is_empty());

    j.as_object_mut().unwrap().remove("type");
    v = CJValidator::from_str(&j.to_string()).unwrap();
    re = v.validate_schema();
    assert!(!re.is_empty());
}

#[test]
fn non_json() {
    let j_mininal = r#"
        {
          "type": "CityJSON",
          "version": "1.1"
          "transform": {
            "scale": [0.0, 0.0, 0.0],
            "translate": [1.0, 1.0, 1.0]
          "CityObjects": {},
          "vertices": []
        }
        "#;
    let re = CJValidator::from_str(&j_mininal);
    assert!(re.is_err());
}
