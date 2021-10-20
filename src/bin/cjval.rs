use ansi_term::Style;
use cjval::CJValidator;

#[macro_use]
extern crate clap;

use std::path::Path;
use url::Url;

use clap::{App, AppSettings, Arg};

use anyhow::{anyhow, Result};

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
    std::process::exit(1);
}

fn main() {
    // Enable ANSI support for Windows
    let desc = format!("{} (supports CityJSON v1.0 + v1.1)", crate_description!());
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
                .help("CityJSON file to validate."),
        )
        .arg(
            Arg::with_name("PATH")
                .short("e")
                .long("extensionfile")
                .multiple(true)
                .takes_value(true)
                .help(
                    "Read the CityJSON Extensions files locally. More than one can \
                     be given. By default the Extension schemas are download, this \
                     overwrites this behaviour",
                ),
        );
    let matches = app.get_matches();

    let p1 = Path::new(matches.value_of("INPUT").unwrap())
        .canonicalize()
        .unwrap();
    let s1 = std::fs::read_to_string(&matches.value_of("INPUT").unwrap())
        .expect("Couldn't read CityJSON file");
    println!(
        "{}",
        Style::new().bold().paint("=== Input CityJSON file ===")
    );
    println!("  {:?}", p1);

    //-- ERRORS
    println!("{}", Style::new().bold().paint("=== JSON syntax ==="));
    let re = CJValidator::from_str(&s1);
    if re.is_err() {
        let s = format!("Invalid JSON file: {:?}", re.as_ref().err().unwrap());
        let e = vec![s];
        print_errors(&e);
        summary_and_bye(-1);
    } else {
        let e: Vec<String> = vec![];
        print_errors(&e);
    }
    let mut val = re.unwrap();

    //-- validate against schema
    println!("{}", Style::new().bold().paint("=== CityJSON schemas ==="));
    let version = val.get_input_cityjson_version();
    match version {
        10 => println!(
            "CityJSON schemas used: v{} (builtin)",
            cjval::CITYJSON_VERSIONS[0]
        ),
        11 => println!(
            "CityJSON schemas used: v{} (builtin)",
            cjval::CITYJSON_VERSIONS[1]
        ),
        _ => {}
    }
    let mut rev = val.validate_schema();
    print_errors(&rev);
    if rev.is_empty() == false {
        summary_and_bye(-1);
    }

    //-- fetch the Extension schemas
    println!(
        "{}",
        Style::new().bold().paint("=== Extensions schemas ===")
    );
    println!("Extension schema(s) used:");
    //-- download them
    if val.get_input_cityjson_version() >= 11 {
        //-- if argument "-e" is passed then do not download
        if let Some(efiles) = matches.values_of("PATH") {
            let l: Vec<&str> = efiles.collect();
            let is_valid = true;
            for s in l {
                let s2 = std::fs::read_to_string(s).expect("Couldn't read Extension file");
                let scanon = Path::new(s).canonicalize().unwrap();
                let re = val.add_one_extension_from_str(&scanon.to_str().unwrap(), &s2);
                match re {
                    Ok(()) => println!("\t- {}.. ok", scanon.to_str().unwrap()),
                    Err(e) => {
                        println!("\t- {}.. ERROR", scanon.to_str().unwrap());
                        println!("\t  ({})", e);
                        summary_and_bye(-1);
                    }
                }
            }
            if is_valid == false {
                summary_and_bye(-1);
            }
        } else {
            //-- download automatically the Extensions
            let re = val.has_extensions();
            if re.is_some() {
                let lexts = re.unwrap();
                // println!("{:?}", lexts);
                for ext in lexts {
                    let o = download_extension(&ext);
                    match o {
                        Ok(l) => {
                            let re = val.add_one_extension_from_str(&ext, &l);
                            match re {
                                Ok(()) => println!("\t- {}.. ok", ext),
                                Err(e) => {
                                    println!("\t- {}.. ERROR", ext);
                                    println!("\t  ({})", e);
                                    summary_and_bye(-1);
                                }
                            }
                        }
                        Err(e) => {
                            println!("\t- {}.. ERROR \n\t{}", ext, e);
                            summary_and_bye(-1);
                        }
                    }
                }
            }
        }
    }
    if val.get_extensions().is_empty() {
        println!("\t- NONE");
    }
    if val.get_input_cityjson_version() == 10 {
        println!("(validation of Extensions is not supported in v1.0, upgrade to v1.1)");
    }
    println!("-----");

    //-- validate Extensions, if any
    rev = val.validate_extensions();
    print_errors(&rev);
    if rev.is_empty() == false {
        summary_and_bye(-1);
    }

    println!(
        "{}",
        Style::new()
            .bold()
            .paint("=== parent_children_consistency ===")
    );
    rev = val.parent_children_consistency();
    print_errors(&rev);
    // if rev.is_empty() == false {
    //     summary_and_bye(-1);
    // }

    println!(
        "{}",
        Style::new().bold().paint("=== wrong_vertex_index ===")
    );
    rev = val.wrong_vertex_index();
    print_errors(&rev);
    // if rev.is_empty() == false {
    //     summary_and_bye(-1);
    // }

    //-- WARNINGS
    let mut bwarns = false;
    if rev.is_empty() == true {
        println!(
            "{}",
            Style::new()
                .bold()
                .paint("=== duplicate_vertices (warnings) ===")
        );
        rev = val.duplicate_vertices();
        print_warnings(&rev);
        if rev.is_empty() == false {
            bwarns = true;
        }
    }

    if rev.is_empty() == true {
        println!(
            "{}",
            Style::new()
                .bold()
                .paint("=== extra_root_properties (warnings) ===")
        );
        rev = val.extra_root_properties();
        print_warnings(&rev);
        if rev.is_empty() == false {
            bwarns = true;
        }
    }

    if rev.is_empty() == true {
        println!(
            "{}",
            Style::new()
                .bold()
                .paint("=== unused_vertices (warnings) ===")
        );
        rev = val.unused_vertices();
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

#[tokio::main]
async fn download_extension(theurl: &str) -> Result<String> {
    // println!("{:?}", jext);
    let u = Url::parse(theurl).unwrap();
    let res = reqwest::get(u).await?;
    if res.status().is_success() {
        Ok(res.text().await?)
    } else {
        return Err(anyhow!("Cannot download extension schema: {}", theurl));
    }
}
