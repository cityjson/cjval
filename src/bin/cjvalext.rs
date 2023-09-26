#[macro_use]
extern crate clap;
use clap::{App, AppSettings, Arg};
use jsonschema::{Draft, JSONSchema};
use serde_json::Value;

static CITYJSON_FILES: [&str; 4] = [
    "cityobjects.schema.json",
    "geomprimitives.schema.json",
    "appearance.schema.json",
    "geomtemplates.schema.json",
];

fn main() {
    // Enable ANSI support for Windows
    let desc = format!("Validation of CityJSON Extension files (supports CityJSON v2.0+v1.1)");
    #[cfg(windows)]
    let _ = ansi_term::enable_ansi_support();
    let app = App::new(crate_name!())
        .setting(AppSettings::ColorAuto)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        // .setting(AppSettings::UnifiedHelpMessage)
        .max_term_width(90)
        .version(crate_version!())
        .about(&*desc)
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .help("CityJSON Extension file (*.ext.json) to validate."),
        );
    let matches = app.get_matches();

    let mut valid = true;

    //-- fetch the instance (the Extension)
    let s1 = std::fs::read_to_string(&matches.value_of("INPUT").unwrap())
        .expect("Couldn't read CityJSON Extension file");
    let re: Result<Value, _> = serde_json::from_str(&s1);
    if re.is_err() {
        valid = false;
        println!("errors: {:?}", re.as_ref().err().unwrap());
    }
    let j: Value = re.unwrap();

    let schema;
    //-- fetch the correct schema
    match j["versionCityJSON"].as_str() {
        Some("1.1") => {
            let schema_str = include_str!("../../schemas/extensions/11/extension.schema.json");
            schema = serde_json::from_str(schema_str).unwrap();
        }
        Some("2.0") => {
            let schema_str = include_str!("../../schemas/extensions/20/extension.schema.json");
            schema = serde_json::from_str(schema_str).unwrap();
        }
        _ => {
            println!("ERROR: the \"versionCityJSON\" property must be \"1.1\" or \"2.0\"");
            println!("❌");
            return;
        }
    }
    // let schema = serde_json::from_str(schema_str).unwrap();
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .expect("A valid schema");
    let result = compiled.validate(&j);
    // let mut ls_errors: Vec<String> = Vec::new();
    if let Err(errors) = result {
        valid = false;
        for error in errors {
            let s: String = format!("{} [path:{}]", error, error.instance_path);
            // ls_errors.push(s);
            println!("ERROR: {}", s);
        }
    }

    //-- validate the URLs and $ref, only a few allowed
    validate_all_ref(&j, &mut valid);

    if valid == true {
        println!("✅");
    } else {
        println!("❌");
    }
}

fn validate_all_ref(j: &Value, valid: &mut bool) {
    if j.is_object() == true {
        let jo = j.as_object().unwrap();
        for p in jo.keys() {
            let tmp = jo.get(p).unwrap();
            if tmp.is_object() {
                validate_all_ref(&tmp, valid);
            }
            if tmp.is_array() {
                let jo = tmp.as_array().unwrap();
                for each in jo {
                    validate_all_ref(&each, valid);
                }
            }
            if tmp.is_string() {
                if p.starts_with("$ref") {
                    let tmp2 = tmp.as_str().unwrap();
                    if tmp2.starts_with("#") == false {
                        let mut b = false;
                        for each in CITYJSON_FILES {
                            if tmp2.starts_with(&each) == true {
                                b = true;
                            }
                        }
                        if b == false {
                            *valid = false;
                            println!("ERROR: {:?} not found.", tmp2);
                        }
                    }
                }
            }
        }
    } else if j.is_array() {
        let jo = j.as_array().unwrap();
        for each in jo {
            validate_all_ref(&each, valid);
        }
    }
}
