use jsonschema::{Draft, JSONSchema};

use structopt::StructOpt;

use serde_json::Value;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    cityjson_file: std::path::PathBuf,
}

fn main() {
    let args = Cli::from_args();

    //-- fetch the CityJSON data file
    let s1 = std::fs::read_to_string(&args.cityjson_file).expect("Couldn't read CityJSON file");
    let j = serde_json::from_str(&s1).unwrap();

    //-- fetch the correct schema
    let schema_str = include_str!("../schemas/cityjson.min.schema.json");
    let schema = serde_json::from_str(schema_str).unwrap();
    // if is_cityjson_file(&j) == false {
    // println!("OUPSIE");
    // }
    let v = get_version_cityjson(&j);
    if v == 10 {
        println!("version {:?}", v);
    } else if v == 11 {
        println!("version {:?}", v);
    } else {
        println!("VERSION NOT SUPPORTED");
    }

    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .expect("A valid schema");
    let result = compiled.validate(&j);
    if let Err(errors) = result {
        for error in errors {
            println!("Validation error: {}", error);
            println!("Instance path: {}", error.instance_path);
        }
    } else {
        println!("valid 👍");
    }
}

fn is_cityjson_file(j: &Value) -> bool {
    if j["type"] == "CityJSON" {
        true
    } else {
        false
    }
}

fn get_version_cityjson(j: &Value) -> i8 {
    if j["version"] == "1.1" {
        11
    } else if j["version"] == 1.0 {
        10
    } else {
        -1
    }
}
