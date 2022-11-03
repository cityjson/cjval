use ansi_term::Style;
use cjval::CJValidator;

#[macro_use]
extern crate clap;

use std::path::Path;
use url::Url;

use clap::{App, AppSettings, Arg};

use anyhow::{anyhow, Result};

fn print_errors(lserrs: &Vec<String>) {
    for (i, e) in lserrs.iter().enumerate() {
        println!("  {}. {}", i + 1, e);
    }
}

fn print_warnings(lswarns: &Vec<String>) {
    for (i, e) in lswarns.iter().enumerate() {
        println!("  {}. {}", i + 1, e);
    }
}

fn summary_and_bye(finalresult: i32) {
    println!("\n");
    println!("============ SUMMARY ============");
    if finalresult == -1 {
        println!("❌ File is invalid");
    } else if finalresult == 0 {
        println!("🟡  File is valid but has warnings");
    } else {
        println!("✅ File is valid");
    }
    println!("=================================");
    std::process::exit(1);
}

fn main() {
    // Enable ANSI support for Windows
    let sversions: Vec<String> = cjval::get_cityjson_schema_all_versions();
    let desc = format!(
        "{}\nSupports CityJSON v1.0 + v1.1 (schemas v{} + v{} are used)",
        crate_description!(),
        sversions[0],
        sversions[1]
    );
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
                     be given. By default the Extension schemas are automatically \
                     downloaded, this overwrites this behaviour",
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

    let mut val = CJValidator::from_str("{}").unwrap();
    //-- ERRORS
    println!("{}", Style::new().bold().paint("=== JSON syntax ==="));
    let re = CJValidator::from_str(&s1);
    match re {
        Ok(f) => {
            println!("ok");
            val = f;
        }
        Err(e) => {
            let s = format!("Invalid JSON file: {:?}", e);
            print_errors(&vec![s]);
            summary_and_bye(-1);
        }
    }

    //-- validate against schema
    println!("{}", Style::new().bold().paint("=== CityJSON schemas ==="));
    if val.get_input_cityjson_version() == 0 {
        println!("CityJSON schemas used: NONE");
    } else {
        println!(
            "CityJSON schemas used: v{} (builtin)",
            val.get_cityjson_schema_version()
        );
    }
    let mut rev = val.validate_schema();
    match rev {
        Ok(_f) => (),
        Err(e) => {
            print_errors(&e);

            summary_and_bye(-1);
        }
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
    match rev {
        Ok(_f) => (),
        Err(e) => {
            print_errors(&e);
            summary_and_bye(-1);
        }
    }

    let mut is_valid = true;

    println!(
        "{}",
        Style::new()
            .bold()
            .paint("=== parent_children_consistency ===")
    );
    rev = val.parent_children_consistency();
    match rev {
        Ok(_f) => println!("ok"),
        Err(e) => {
            print_errors(&e);
            is_valid = false;
        }
    }

    println!(
        "{}",
        Style::new().bold().paint("=== wrong_vertex_index ===")
    );
    rev = val.wrong_vertex_index();
    match rev {
        Ok(_f) => println!("ok"),
        Err(e) => {
            print_errors(&e);
            is_valid = false;
        }
    }

    println!("{}", Style::new().bold().paint("=== semantics_arrays ==="));
    rev = val.semantics_arrays();
    match rev {
        Ok(_f) => println!("ok"),
        Err(e) => {
            print_errors(&e);
            is_valid = false;
        }
    }

    //-- if not valid at this point then stop
    if is_valid == false {
        summary_and_bye(-1);
    }

    //-- WARNINGS
    let mut bwarns = false;
    println!(
        "{}",
        Style::new()
            .bold()
            .paint("=== duplicate_vertices (warnings) ===")
    );
    rev = val.duplicate_vertices();
    match rev {
        Ok(_f) => println!("ok"),
        Err(e) => {
            print_warnings(&e);
            bwarns = true;
        }
    }

    if bwarns == false {
        println!(
            "{}",
            Style::new()
                .bold()
                .paint("=== extra_root_properties (warnings) ===")
        );
        rev = val.extra_root_properties();
        match rev {
            Ok(_f) => println!("ok"),
            Err(e) => {
                print_warnings(&e);
                bwarns = true;
            }
        }
    }

    if bwarns == false {
        println!(
            "{}",
            Style::new()
                .bold()
                .paint("=== unused_vertices (warnings) ===")
        );
        rev = val.unused_vertices();
        match rev {
            Ok(_f) => println!("ok"),
            Err(e) => {
                print_warnings(&e);
                bwarns = true;
            }
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
