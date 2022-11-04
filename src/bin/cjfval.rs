// use ansi_term::Style;
use cjval::CJValidator;

#[macro_use]
extern crate clap;

use url::Url;

use std::io;
use std::io::BufRead;

use clap::{App, AppSettings, Arg};

use anyhow::{anyhow, Result};

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

    let mut b_verbose = false;
    if matches.occurrences_of("verbose") > 0 {
        b_verbose = true;
    }
    let mut b_metadata = false;
    let mut val = CJValidator::from_str("{}").unwrap();
    let stdin = std::io::stdin();
    for (i, line) in stdin.lock().lines().enumerate() {
        let l = line.unwrap();
        if l.is_empty() {
            continue;
        }
        if !b_metadata {
            let tmp = CJValidator::from_str(&l);
            match tmp {
                Ok(f) => {
                    val = f;
                    let re = validate_cj(&mut val);
                    match re {
                        Ok(_) => {
                            let w = validate_cj_warnings(&val);
                            match w {
                                Ok(_) => println!("l.{}\t✅", i + 1),
                                Err(e) => {
                                    println!("l.{}\t🟡", i + 1);
                                    if b_verbose {
                                        println!("{}", e.join(" | "));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("l.{}\t❌", i + 1);
                            if b_verbose {
                                println!("{}", e.join(" | "));
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("l.{}\t❌", i + 1);
                    if b_verbose {
                        let s = format!("Invalid JSON file: {}", e);
                        println!("{}", s);
                    }
                }
            }
            b_metadata = true;
        } else {
            let re = val.replace_cjfeature(&l);
            match re {
                Ok(_) => {
                    let re = validate_cjf(&val);
                    match re {
                        Ok(_) => {
                            let w = validate_cjf_warnings(&val);
                            match w {
                                Ok(_) => println!("l.{}\t✅", i + 1),
                                Err(e) => {
                                    println!("l.{}\t🟡", i + 1);
                                    if b_verbose {
                                        println!("{}", e.join(" | "));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("l.{}\t❌", i + 1);
                            if b_verbose {
                                println!("{}", e.join(" | "));
                            }
                        }
                    }
                    // println!("{} ok", i);
                }
                Err(e) => {
                    println!("l.{}\t❌", i + 1);
                    if b_verbose {
                        let e = format!("Invalid JSON file: {:?}", e);
                        println!("{}", e);
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_cj(val: &mut CJValidator) -> Result<(), Vec<String>> {
    let mut b_valid = true;
    let mut ls_errors: Vec<String> = Vec::new();
    //-- CityJSON schema
    let re = val.validate_schema();
    match re {
        Ok(_) => (),
        Err(errors) => {
            b_valid = false;
            for error in errors {
                let s: String = format!("{}", error);
                ls_errors.push(s);
            }
        }
    }
    //-- Extensions
    if val.get_input_cityjson_version() >= 11 {
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
                            Ok(_) => (),
                            Err(error) => {
                                b_valid = false;
                                let s: String = format!("{}", error);
                                ls_errors.push(s);
                            }
                        }
                    }
                    Err(error) => {
                        let s: String = format!("{}", error);
                        ls_errors.push(s);
                        b_valid = false;
                    }
                }
            }
        }
    }
    if b_valid {
        Ok(())
    } else {
        Err(ls_errors)
    }
}

fn validate_cj_warnings(val: &CJValidator) -> Result<(), Vec<String>> {
    let _ = val.extra_root_properties()?;
    Ok(())
}

fn validate_cjf(val: &CJValidator) -> Result<(), Vec<String>> {
    let _ = val.validate_schema()?;
    let _ = val.validate_extensions()?;
    let _ = val.parent_children_consistency()?;
    let _ = val.wrong_vertex_index()?;
    let _ = val.semantics_arrays()?;
    Ok(())
}

fn validate_cjf_warnings(val: &CJValidator) -> Result<(), Vec<String>> {
    let _ = val.duplicate_vertices()?;
    let _ = val.unused_vertices()?;
    Ok(())
}
