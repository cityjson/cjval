use ansi_term::Colour::Red;
use ansi_term::Style;
use cjval::CJValidator;

extern crate clap;

use std::collections::HashMap;
use std::path::PathBuf;
use url::Url;

use clap::Parser;

use anyhow::{anyhow, Result};

#[derive(Parser)]
#[command(version, about = "Validation of a CityJSON file", long_about = None)]
struct Cli {
    /// CityJSON input file
    inputfile: Option<PathBuf>,
    #[arg(short, long)]
    verbose: bool,
    /// Read the CityJSON Extensions files locally instead of downloading them.
    /// More than one can be given.
    #[arg(short, long)]
    extensionfiles: Vec<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    match cli.inputfile {
        Some(ifile) => {
            if !ifile.exists() {
                eprintln!("ERROR: Input file {} doesn't exist", ifile.display());
                std::process::exit(0);
            }
            if let Some(ext) = ifile.extension() {
                if ext != "json" && ext != "jsonl" {
                    eprintln!(
                        "ERROR: file extension {} not supported (only .json and .jsonl)",
                        ext.to_str().unwrap()
                    );
                    std::process::exit(0);
                }
            }
            process_cityjson_file(&ifile, &cli.extensionfiles, cli.verbose);
        }
        None => {
            println!("stdin input");
        }
    }
}

fn summary_and_bye(finalresult: i32, verbose: bool) {
    if verbose {
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
    } else {
        if finalresult == -1 {
            println!("❌ invalid");
        } else if finalresult == 0 {
            println!("🟡 has warnings");
        } else {
            println!("✅ valid");
        }
    }
    std::process::exit(0);
}

// fn process_cjseq_stream(verbose: bool) {
//     let mut b_metadata = false;
//     let mut val = CJValidator::from_str("{}");
//     let stdin = std::io::stdin();
//     for (i, line) in stdin.lock().lines().enumerate() {
//         let l = line.unwrap();
//         if l.is_empty() {
//             continue;
//         }
//         if !b_metadata {
//             // TODO: what if no metadata-first-line?
//             val = CJValidator::from_str(&l);
//             let re = fetch_extensions(&mut val, &cli.extensionfiles);
//             match re {
//                 Ok(_) => {
//                     let valsumm = val.validate();
//                     let status = get_status(&valsumm);
//                     match status {
//                         1 => println!("l.{}\t✅", i + 1),
//                         0 => {
//                             println!("l.{}\t🟡", i + 1);
//                             if b_verbose {
//                                 println!("{}", get_errors_string(&valsumm));
//                             }
//                         }
//                         -1 => {
//                             println!("l.{}\t❌", i + 1);
//                             if b_verbose {
//                                 println!("{}", get_errors_string(&valsumm));
//                             }
//                         }
//                         _ => (),
//                     }
//                 }
//                 Err(e) => {
//                     println!("l.{}\t❌", i + 1);
//                     if b_verbose {
//                         println!("{}", e.join(" | "));
//                     }
//                 }
//             }
//             b_metadata = true;
//         } else {
//             let re = val.from_str_cjfeature(&l);
//             match re {
//                 Ok(_) => {
//                     let valsumm = val.validate();
//                     let status = get_status(&valsumm);
//                     match status {
//                         1 => println!("l.{}\t✅", i + 1),
//                         0 => {
//                             if b_verbose {
//                                 println!("l.{}\t🟡\t{}", i + 1, get_errors_string(&valsumm));
//                             } else {
//                                 println!("l.{}\t🟡", i + 1);
//                             }
//                         }
//                         -1 => {
//                             if b_verbose {
//                                 println!("l.{}\t❌\t{}", i + 1, get_errors_string(&valsumm));
//                             } else {
//                                 println!("l.{}\t❌", i + 1);
//                             }
//                         }
//                         _ => (),
//                     }
//                 }
//                 Err(e) => {
//                     if b_verbose {
//                         println!("l.{}\t❌\t{}", i + 1, format!("Invalid JSON file: {:?}", e));
//                     } else {
//                         println!("l.{}\t❌", i + 1);
//                     }
//                 }
//             }
//         }
//     }
// }

fn process_cityjson_file(ifile: &PathBuf, extpaths: &Vec<PathBuf>, verbose: bool) {
    let p1 = ifile.canonicalize().unwrap();
    let s1 = std::fs::read_to_string(&p1).expect("Couldn't read CityJSON file");

    if verbose {
        println!(
            "{}",
            Style::new().bold().paint("=== Input CityJSON file ===")
        );
        println!("{:?}", p1);
    }

    //-- Get the validator
    let mut val = CJValidator::from_str(&s1);

    //-- print the schema version used
    if verbose {
        println!("{}", Style::new().bold().paint("=== CityJSON schemas ==="));
        if val.get_input_cityjson_version() == 0 {
            println!("none");
        } else {
            println!("v{} (builtin)", val.get_cityjson_schema_version());
        }
    }

    //-- Extensions
    if verbose {
        println!("{}", Style::new().bold().paint("=== Extensions ==="));
    }
    let re = fetch_extensions(&mut val, &extpaths);
    match re {
        Ok(x) => {
            if verbose {
                for (ext, s) in &x {
                    println!(" - {ext}... {s}");
                }
                if x.is_empty() {
                    println!("none");
                }
            }
        }
        Err(x) => {
            if verbose {
                for (ext, s) in &x {
                    println!(" - {ext}... {s}");
                }
            }
            summary_and_bye(-1, verbose);
        }
    }

    let valsumm = val.validate();
    let mut has_errors = false;
    let mut has_warnings = false;
    if verbose {
        for (criterion, summ) in valsumm.iter() {
            println!(
                "{} {} {} ",
                Style::new().bold().paint("==="),
                Style::new().bold().paint(criterion),
                Style::new().bold().paint("===")
            );
            println!("{}", summ);
            if summ.has_errors() == true {
                if summ.is_warning() == true {
                    has_warnings = true;
                } else {
                    has_errors = true;
                }
            }
        }
    }

    //-- bye-bye
    if has_errors == false && has_warnings == false {
        summary_and_bye(1, verbose);
    } else if has_errors == false && has_warnings == true {
        summary_and_bye(0, verbose);
    } else {
        summary_and_bye(-1, verbose);
    }
}

fn fetch_extensions(
    val: &mut CJValidator,
    extpaths: &Vec<PathBuf>,
) -> Result<HashMap<String, String>, HashMap<String, String>> {
    let mut b_valid = true;
    let mut d_errors: HashMap<String, String> = HashMap::new();
    // let mut ls_errors: Vec<String> = Vec::new();
    //-- Extensions
    // if val.get_input_cityjson_version() == 10 && verbose {
    //     println!("(validation of Extensions is not supported in CityJSON v1.0, upgrade to v1.1)");
    // }
    if val.get_input_cityjson_version() >= 11 {
        //-- if argument "-e" is passed then do not download
        if extpaths.len() > 0 {
            for fext in extpaths {
                let s = format!("{}", fext.to_str().unwrap());
                d_errors.insert(s, "ok".to_string());
            }
            for fext in extpaths {
                let sf = format!("{}", fext.to_str().unwrap());
                if fext.exists() {
                    let fexts =
                        std::fs::read_to_string(&fext).expect("Couldn't read Extension file");
                    let re = val.add_one_extension_from_str(&fexts);
                    match re {
                        Ok(_) => (),
                        Err(e) => {
                            let s = format!(
                                "Error with Extension file: {} ({})",
                                fext.to_str().unwrap(),
                                e
                            );
                            if let Some(x) = d_errors.get_mut(&sf) {
                                *x = s;
                            }
                            b_valid = false;
                        }
                    }
                } else {
                    let s = format!("Extension file: {} doesn't exist", fext.to_str().unwrap());
                    if let Some(x) = d_errors.get_mut(&sf) {
                        *x = s;
                    }

                    b_valid = false;
                    // summary_and_bye(-1, verbose);
                }
            }
        } else {
            //-- download automatically the Extensions
            let re = val.get_extensions_urls();
            if re.is_some() {
                let lexts = re.unwrap();
                // println!("{:?}", lexts);
                for ext in &lexts {
                    let s = format!("{}", ext);
                    d_errors.insert(s, "ok".to_string());
                }
                for ext in lexts {
                    let s2 = format!("{}", ext);
                    let o = download_extension(&ext);
                    match o {
                        Ok(l) => {
                            let re = val.add_one_extension_from_str(&l);
                            match re {
                                Ok(_) => (),
                                Err(error) => {
                                    b_valid = false;
                                    let s: String = format!("{}", error);
                                    // ls_errors.push(s);
                                    if let Some(x) = d_errors.get_mut(&s2) {
                                        *x = s;
                                    }
                                }
                            }
                        }
                        Err(error) => {
                            let s: String = format!("{}", error);
                            if let Some(x) = d_errors.get_mut(&s2) {
                                *x = s;
                            }
                            b_valid = false;
                        }
                    }
                }
            }
        }
    }
    if b_valid {
        Ok(d_errors)
    } else {
        Err(d_errors)
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
