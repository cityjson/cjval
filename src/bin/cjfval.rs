use ansi_term::Style;
use cjval::CJValidator;

#[macro_use]
extern crate clap;

use std::path::Path;
use url::Url;

use std::io::BufRead;
use std::io::{self, Write};

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
    println!("============= SUMMARY =============");
    if finalresult == -1 {
        println!("❌ File is invalid");
    } else if finalresult == 0 {
        println!("🟡  File is valid but has warnings");
    } else {
        println!("✅ File is valid");
    }
    println!("===================================");
    std::process::exit(1);
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

fn main() -> io::Result<()> {
    // Enable ANSI support for Windows
    let sversions: Vec<String> = cjval::get_cityjson_schema_all_versions();
    let desc = format!(
        "{}\nSupports CityJSONFeature v1.1 (schemas v{} are used)",
        "Validation of CityJSONFeature streams (JSONL)", sversions[1]
    );
    #[cfg(windows)]
    let _ = ansi_term::enable_ansi_support();
    let app = App::new("cjfval")
        .setting(AppSettings::ColorAuto)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        // .setting(AppSettings::UnifiedHelpMessage)
        .max_term_width(90)
        .version(crate_version!())
        .about(&*desc)
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .multiple(true)
                .help("Explain in detail the errors and warnings"),
        );
    let matches = app.get_matches();

    let mut bMetadata = false;
    let mut val = CJValidator::from_str("{}").unwrap();
    let stdin = std::io::stdin();
    for (i, line) in stdin.lock().lines().enumerate() {
        let l = line.unwrap();
        if l.is_empty() {
            continue;
        }
        if !bMetadata {
            let tmp = CJValidator::from_str(&l);
            match tmp {
                Ok(f) => {
                    val = f;
                    let re = validate_cj(&mut val);
                    println!("{} {:?}", i + 1, re);
                }
                Err(e) => {
                    let s = format!("Invalid JSON file: {:?}", e);
                    println!("{} {}", i + 1, s);
                }
            }
            bMetadata = true;
        } else {
            let re = val.replace_cjfeature(&l);
            match re {
                Ok(_) => {
                    let re = validate_cjf(&val);
                    println!("{} ok", i);
                }
                Err(e) => {
                    let s = format!("Invalid JSON file: {:?}", e);
                    println!("{} {}", i + 1, s);
                }
            }
        }
    }
    Ok(())
}

fn validate_cj(val: &mut CJValidator) -> Result<(), String> {
    let mut bValid = true;
    //-- CityJSON schema
    let re = val.validate_schema();
    match re {
        Ok(_f) => (),
        Err(e) => {
            println!("schema invalid");
            bValid = false;
        }
    }
    //-- Extensions
    if val.get_input_cityjson_version() >= 11 {
        //-- download automatically the Extensions
        let re = val.has_extensions();
        if re.is_some() {
            let lexts = re.unwrap();
            println!("{:?}", lexts);
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
                                // summary_and_bye(-1);
                            }
                        }
                    }
                    Err(e) => {
                        println!("\t- {}.. ERROR \n\t{}", ext, e);
                    }
                }
            }
        }
    }
    //-- warnings
    let rev = val.extra_root_properties();
    println!("{:?}", rev);
    Ok(())
}

fn validate_cjf(val: &CJValidator) -> Result<(), Vec<String>> {
    // let mut bErrors = false;
    // let mut bWarnings = false;
    let _ = val.validate_schema()?;
    let _ = val.validate_extensions()?;
    let _ = val.parent_children_consistency()?;
    let _ = val.wrong_vertex_index()?;
    let _ = val.semantics_arrays()?;
    //-- warnings
    let redv = val.duplicate_vertices();
    let reuv = val.unused_vertices();
    // println!("{:?}", rev);
    Ok(())
}
