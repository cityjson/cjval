use cjval::CJValidator;
use std::process;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    cityjson_file: std::path::PathBuf,

    #[structopt(short = "e", long = "extensionfile", parse(from_os_str))]
    extensions: Vec<std::path::PathBuf>,
}

fn print_errors(lserrs: &Vec<String>) {
    if lserrs.is_empty() {
        println!("ok");
    } else {
        for (i, e) in lserrs.iter().enumerate() {
            println!("  {}. {}", i + 1, e);
        }
    }
}

fn print_warnings(lswarns: &Vec<String>) {
    if lswarns.is_empty() {
        println!("ok");
    } else {
        for (i, e) in lswarns.iter().enumerate() {
            println!("  {}. {}", i + 1, e);
        }
    }
}

fn summary_and_bye(finalresult: i32) {
    println!("\n");
    println!("============ SUMMARY ============");
    if finalresult == -1 {
        println!("❌ File is invalid");
    } else if finalresult == 0 {
        println!("⚠️  File is valid but has warnings");
    } else {
        println!("✅ File is valid");
    }
    println!("=================================");
    process::exit(0x0100);
}

fn main() {
    let args = Cli::from_args();

    //-- fetch the CityJSON data file
    let s1 = std::fs::read_to_string(&args.cityjson_file).expect("Couldn't read CityJSON file");
    //-- fetch the Extension schemas
    let mut exts: Vec<String> = Vec::new();
    for ext in args.extensions {
        let s2 = std::fs::read_to_string(ext).expect("Couldn't read Extension file");
        exts.push(s2);
    }

    //-- ERRORS
    println!("=== JSON syntax ===");
    let re = CJValidator::from_str(&s1, &exts);
    if re.is_err() {
        let s = format!("Invalid JSON file: {:?}", re.as_ref().err().unwrap());
        let e = vec![s];
        print_errors(&e);
        summary_and_bye(-1);
    } else {
        let e: Vec<String> = vec![];
        print_errors(&e);
    }
    let val = re.unwrap();

    //-- validate against schema
    println!("=== CityJSON schemas ===");
    let mut rev = val.validate_schema();
    print_errors(&rev);
    if rev.is_empty() == false {
        summary_and_bye(-1);
    }

    //-- validate Extensions, if any
    println!("=== Extensions schemas ===");
    rev = val.validate_extensions();
    print_errors(&rev);
    if rev.is_empty() == false {
        summary_and_bye(-1);
    }

    println!("=== parent_children_consistency ===");
    rev = val.parent_children_consistency();
    print_errors(&rev);
    if rev.is_empty() == false {
        summary_and_bye(-1);
    }

    println!("=== wrong_vertex_index ===");
    rev = val.wrong_vertex_index();
    print_errors(&rev);
    if rev.is_empty() == false {
        summary_and_bye(-1);
    }

    //-- WARNINGS
    let mut bwarns = false;
    if rev.is_empty() == true {
        println!("=== duplicate_vertices ===");
        rev = val.duplicate_vertices();
        print_warnings(&rev);
        if rev.is_empty() == false {
            bwarns = true;
        }
    }

    if rev.is_empty() == true {
        println!("=== extra_root_properties ===");
        rev = val.extra_root_properties();
        print_warnings(&rev);
        if rev.is_empty() == false {
            bwarns = true;
        }
    }

    //-- bye-bye
    if bwarns == false {
        summary_and_bye(1);
    } else {
        summary_and_bye(0);
    }
}
