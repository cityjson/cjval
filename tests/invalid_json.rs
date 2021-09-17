use crate::cjval::CJValidator;
use cjval;
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
    let mut v: CJValidator = CJValidator::from_str(&j.to_string());
    let re = v.validate_schema();
    println!("{:?}", re);
    assert!(re.is_empty());
}
