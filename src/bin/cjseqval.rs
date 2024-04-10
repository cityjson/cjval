use cjval::CJValidator;
use cjval::ValSummary;
use indexmap::IndexMap;
use std::path::PathBuf;

extern crate clap;
use anyhow::{anyhow, Result};
use clap::Parser;
use std::fmt::Write;
use std::io;
use std::io::BufRead;
use url::Url;

#[derive(Parser)]
#[command(version, about = "Validation of a CityJSONSeq", long_about = None)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
    /// Read the CityJSON Extensions files locally instead of downloading them.
    /// More than one can be given.
    #[arg(short, long)]
    extensionfiles: Vec<PathBuf>,
}

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
    let cli = Cli::parse();
    let b_verbose = cli.verbose;
    let mut b_metadata = false;
    let mut val = CJValidator::from_str("{}");
    let stdin = std::io::stdin();
    for (i, line) in stdin.lock().lines().enumerate() {
        let l = line.unwrap();
        if l.is_empty() {
            continue;
        }
        if !b_metadata {
            // TODO: what if no metadata-first-line?
            val = CJValidator::from_str(&l);
            let re = fetch_extensions(&mut val, &cli.extensionfiles);
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

fn fetch_extensions(val: &mut CJValidator, extpaths: &Vec<PathBuf>) -> Result<(), Vec<String>> {
    let mut b_valid = true;
    let mut ls_errors: Vec<String> = Vec::new();
    //-- Extensions
    if val.get_input_cityjson_version() >= 11 {
        if extpaths.len() > 0 {
            for s in extpaths {
                if s.exists() {
                    let s2 = std::fs::read_to_string(&s).expect("Couldn't read Extension file");
                    let re = val.add_one_extension_from_str(&s2);
                    match re {
                        Ok(_) => (),
                        Err(e) => {
                            let s = format!(
                                "Error with Extension file: {} ({})",
                                s.to_str().unwrap(),
                                e
                            );
                            ls_errors.push(s);
                            b_valid = false;
                        }
                    }
                } else {
                    let s = format!("Extension file: {} doesn't exist", s.to_str().unwrap());
                    ls_errors.push(s);
                    b_valid = false;
                }
            }
        } else {
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
    if b_valid {
        Ok(())
    } else {
        Err(ls_errors)
    }
}
