use structopt::StructOpt;

use cjval::CJValidator;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    cityjson_file: std::path::PathBuf,
}

fn main() {
    let args = Cli::from_args();

    //-- fetch the CityJSON data file
    let s1 = std::fs::read_to_string(&args.cityjson_file).expect("Couldn't read CityJSON file");
    let re = CJValidator::from_str(&s1);
    if re.is_err() {
        println!("Invalid JSON file: {:?}", re.as_ref().err().unwrap());
        return;
    }
    let val = re.unwrap();

    //-- validate against schema
    println!("=== validate against schema ===");
    let mut rev = val.validate_schema();
    if rev.is_empty() {
        println!("\tVALID :)");
    } else {
        println!("\t==INVALID==");
        for (i, e) in rev.iter().enumerate() {
            println!("\t{}. {:?}", i + 1, e);
        }
    }
    // println!("{:?}", re2);

    if rev.is_empty() == true {
        println!("=== parent_children_consistency ===");
        rev = val.parent_children_consistency();
        println!("{:?}", rev);
    }
    if rev.is_empty() == true {
        println!("=== duplicate_vertices ===");
        let re = val.duplicate_vertices();
        println!("{:?}", re);
    }
}
