// use ansi_term::Style;
use cjval::CJValidator;
use cjval::ValSummary;
use indexmap::IndexMap;
#[macro_use]
extern crate clap;
use anyhow::{anyhow, Result};
use clap::{App, AppSettings, Arg, Values};
use std::fmt::Write;
use std::io;
use std::io::BufRead;
use std::path::Path;
use url::Url;

#[tokio::main]
async fn download_extension(theurl: &str) -> Result<String> {
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
        "{}\nSupports CityJSONFeature v2.0+v1.1 (schemas v{} + v{} are used)",
        "Validation of CityJSONFeature streams (JSONL)", sversions[2], sversions[1]
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
    // let extfiles = matches.values_of("PATH");

    let mut b_verbose = false;
    if matches.occurrences_of("verbose") > 0 {
        b_verbose = true;
    }
    let mut b_metadata = false;
    let mut val = CJValidator::from_str("{}");
    let stdin = std::io::stdin();
    for (i, line) in stdin.lock().lines().enumerate() {
        let l = line.unwrap();
        if l.is_empty() {
            continue;
        }
        if !b_metadata {
            // TODO: what is no metadata-first-line?
            val = CJValidator::from_str(&l);
            let re = fetch_extensions(&mut val, matches.values_of("PATH"));
            match re {
                Ok(_) => {
                    let valsumm = val.validate();
                    let status = get_status(&valsumm);
                    match status {
                        1 => println!("l.{}\t✅", i + 1),
                        0 => {
                            println!("l.{}\t🟡", i + 1);
                            if b_verbose {
                                println!("{}", get_errors_string(&valsumm));
                            }
                        }
                        -1 => {
                            println!("l.{}\t❌", i + 1);
                            if b_verbose {
                                println!("{}", get_errors_string(&valsumm));
                            }
                        }
                        _ => (),
                    }
                }
                Err(e) => {
                    println!("l.{}\t❌", i + 1);
                    if b_verbose {
                        println!("{}", e.join(" | "));
                    }
                }
            }
            b_metadata = true;
        } else {
            let re = val.from_str_cjfeature(&l);
            match re {
                Ok(_) => {
                    let valsumm = val.validate();
                    let status = get_status(&valsumm);
                    match status {
                        1 => println!("l.{}\t✅", i + 1),
                        0 => {
                            if b_verbose {
                                println!("l.{}\t🟡\t{}", i + 1, get_errors_string(&valsumm));
                            } else {
                                println!("l.{}\t🟡", i + 1);
                            }
                        }
                        -1 => {
                            if b_verbose {
                                println!("l.{}\t❌\t{}", i + 1, get_errors_string(&valsumm));
                            } else {
                                println!("l.{}\t❌", i + 1);
                            }
                        }
                        _ => (),
                    }
                }
                Err(e) => {
                    if b_verbose {
                        println!("l.{}\t❌\t{}", i + 1, format!("Invalid JSON file: {:?}", e));
                    } else {
                        println!("l.{}\t❌", i + 1);
                    }
                }
            }
        }
    }
    Ok(())
}

fn get_status(valsumm: &IndexMap<String, ValSummary>) -> i8 {
    let mut has_errors = false;
    let mut has_warnings = false;
    for (_criterion, summ) in valsumm.iter() {
        if summ.has_errors() == true {
            if summ.is_warning() == true {
                has_warnings = true;
            } else {
                has_errors = true;
            }
        }
    }
    if has_errors == false && has_warnings == false {
        1
    } else if has_errors == false && has_warnings == true {
        0
    } else {
        -1
    }
}

fn get_errors_string(valsumm: &IndexMap<String, ValSummary>) -> String {
    let mut s = String::new();
    for (_criterion, summ) in valsumm.iter() {
        if summ.has_errors() == true {
            write!(&mut s, "{} | ", summ).expect("Problem writing String");
        }
    }
    s
}

fn fetch_extensions(val: &mut CJValidator, extpaths: Option<Values>) -> Result<(), Vec<String>> {
    let mut b_valid = true;
    let mut ls_errors: Vec<String> = Vec::new();
    //-- Extensions
    if val.get_input_cityjson_version() >= 11 {
        match extpaths {
            Some(efiles) => {
                let l: Vec<&str> = efiles.collect();
                for s in l {
                    let s2 = std::fs::read_to_string(s).expect("Couldn't read Extension file");
                    let scanon = Path::new(s).canonicalize().unwrap();
                    let re = val.add_one_extension_from_str(&s2);
                    match re {
                        Ok(_) => (),
                        Err(e) => {
                            let s = format!(
                                "Error with Extension file: {} ({})",
                                scanon.to_str().unwrap(),
                                e
                            );
                            ls_errors.push(s);
                            b_valid = false;
                        }
                    }
                }
            }
            None => {
                //-- download automatically the Extensions
                let re = val.get_extensions_urls();
                if re.is_some() {
                    let lexts = re.unwrap();
                    // println!("{:?}", lexts);
                    for ext in lexts {
                        let o = download_extension(&ext);
                        match o {
                            Ok(l) => {
                                let re = val.add_one_extension_from_str(&l);
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
        }
    }
    if b_valid {
        Ok(())
    } else {
        Err(ls_errors)
    }
}
