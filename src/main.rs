use structopt::StructOpt;

use cjval::CJValidator;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    cityjson_file: std::path::PathBuf,

    #[structopt(short = "e", long = "extensionfile", parse(from_os_str))]
    extensions: Vec<std::path::PathBuf>,
}

fn print_errors(lserrs: &Vec<String>) {
    if lserrs.is_empty() {
        println!("👍🏼");
    } else {
        println!("❌");
        for (i, e) in lserrs.iter().enumerate() {
            println!("\t{}. {}", i + 1, e);
        }
    }
}

fn main() {
    let args = Cli::from_args();
    // println!("{:?}", args.cityjson_file);
    // println!("{:?}", args.extensions);
    // return;

    //-- fetch the CityJSON data file
    let s1 = std::fs::read_to_string(&args.cityjson_file).expect("Couldn't read CityJSON file");
    let re = CJValidator::from_str(&s1);
    if re.is_err() {
        println!("Invalid JSON file: {:?}", re.as_ref().err().unwrap());
        return;
    }
    let mut val = re.unwrap();

    //-- validate against schema
    println!("=== validate against schema ===");
    let mut rev = val.validate_schema();
    print_errors(&rev);

    //-- validate Extensions, if any
    if rev.is_empty() == true {
        println!("=== validate Extensions ===");
        let exts: Vec<String> = val.contains_extensions();
        if exts.is_empty() == false {
            for ext in args.extensions {
                let s1 = std::fs::read_to_string(ext).expect("Couldn't read Extension file");
                val.add_extension(&s1);
            }
        }
    }

    if rev.is_empty() == true {
        println!("=== parent_children_consistency ===");
        rev = val.parent_children_consistency();
        print_errors(&rev);
    }

    if rev.is_empty() == true {
        println!("=== wrong vertex index ===");
        rev = val.wrong_vertex_index();
        print_errors(&rev);
    }

    //-- warnings
    if rev.is_empty() == true {
        println!("=== duplicate_vertices ===");
        rev = val.duplicate_vertices();
        print_errors(&rev);
    }
}
