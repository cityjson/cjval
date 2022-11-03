use crate::cjval::CJValidator;
use cjval;

use serde_json::Value;

fn get_data() -> Value {
    let j_mininal = r#"
        {
            "type": "CityJSON",
            "version": "1.1",
            "CityObjects":
            {
                "LondonTower": { "type": "Building" },
                "LondonTower": { "type": "WaterBody" }
            },
            "vertices": [
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
fn contains_same_key() {
    let j = get_data();
    let v: CJValidator = CJValidator::from_str(&j.to_string()).unwrap();
    let re = v.validate_schema();
    assert!(re.is_ok());
}
