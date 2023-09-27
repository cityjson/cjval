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

#[test]
fn extension_generic() {
    let mut j = get_minimal();
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

// #[test]
// fn extension_reuse_cityobjects() {
//     let sdata = std::fs::read_to_string("data/potatoes.city.json").unwrap();
//     let mut v: CJValidator = CJValidator::from_str(&sdata);
//     let re = v.validate();
//     assert!(re.is_ok());

//     let sschema = std::fs::read_to_string("schemas/extensions/potato.ext.json").unwrap();
//     let _ = v.add_one_extension_from_str(&"Noise".to_string(), &sschema);
//     let rev = v.validate_extensions();
//     assert!(rev.is_ok());
// }
