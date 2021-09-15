use serde_json::Value;

struct CJValidator {
    j: Value,
    is_cityjson: bool,
    version: i32,
    is_schema_valid: i32,
}

impl CJValidator {
    fn from_str(&self, s: &str) -> Result<Self, String> {
        let re = serde_json::from_str(&s);
        //-- is it a valid JSON file?
        if re.is_err() {
            return Err("Not a valid JSON file".to_string());
        }
        let j: Value = re.unwrap();
        //-- is it a CityJSON file?
        if j["type"] != "CityJSON" {
            return Err("Not a CityJSON file".to_string());
        }
        //-- which cityjson version
        let mut v: i32 = -1;
        if j["version"] == "1.1" {
            v = 11;
        } else if j["version"] == 1.0 {
            v = 10;
        }
        Ok(CJValidator {
            j: j,
            is_cityjson: true,
            version: v,
            is_schema_valid: -1,
        })
    }
}
