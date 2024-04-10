extern crate clap;
use clap::Parser;
use std::path::PathBuf;

use jsonschema::{Draft, JSONSchema};
use serde_json::Value;

#[derive(Parser)]
#[command(version, about = "Validation of a CityJSON Extension file", long_about = None)]
struct Cli {
    /// CityJSON Extension file
    inputfile: PathBuf,
}

static CITYJSON_FILES: [&str; 4] = [
    "cityobjects.schema.json",
    "geomprimitives.schema.json",
    "appearance.schema.json",
    "geomtemplates.schema.json",
];

fn main() {
    let cli = Cli::parse();
    let mut valid = true;
    if !cli.inputfile.exists() {
        eprintln!(
            "ERROR: Input file {} doesn't exist",
            cli.inputfile.display()
        );
        std::process::exit(0);
    }
    //-- fetch the instance (the Extension)
    let p1 = cli.inputfile.canonicalize().unwrap();
    let s1 = std::fs::read_to_string(&p1).expect("Couldn't read the file");
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
    std::process::exit(0);
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
