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
    let mut v: CJValidator = CJValidator::from_str(&s1);

    //-- validate against schema
    println!("=== validate against schema ===");
    let re = v.validate_schema();
    if re.is_empty() {
        println!("\tVALID :)");
    } else {
        println!("\t==INVALID==");
        for (i, e) in re.iter().enumerate() {
            println!("\t{}. {:?}", i + 1, e);
        }
    }
    // println!("{:?}", re);

    if re.is_empty() == true {
        println!("=== parent_children_consistency ===");
        let re = v.parent_children_consistency();
        println!("{:?}", re);
    }
}
