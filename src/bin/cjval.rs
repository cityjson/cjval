use ansi_term::Style;
use cjval::CJValidator;
use cjval::ValSummary;
use indexmap::IndexMap;

extern crate clap;

use std::collections::HashMap;
use std::fmt::Write as fmtwrite;
use std::io::BufRead;
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
            let fext = ifile.extension().unwrap().to_str().unwrap();
            match fext {
                "json" | "JSON" => process_cityjson_file(&ifile, &cli.extensionfiles, cli.verbose),
                _ => {
                    eprintln!("ERROR: file extension .{} not supported (only .json)", fext);
                    std::process::exit(0);
                }
            }
        }
        None => {
            let _ = process_cjseq_stream(&cli.extensionfiles, cli.verbose);
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

fn process_cjseq_stream(extpaths: &Vec<PathBuf>, verbose: bool) {
    let mut b_metadata = false;
    let mut val = CJValidator::from_str("{}");
    let stdin = std::io::stdin();
    let mut finalresult: i8 = 1;
    let mut linetotal: u64 = 0;
    for (i, line) in stdin.lock().lines().enumerate() {
        let l = line.unwrap();
        if l.is_empty() {
            continue;
        }
        linetotal += 1;
        if !b_metadata {
            val = CJValidator::from_str(&l);
            if val.is_cityjson() == false {
                //-- therefore not a CityJSON first line
                println!("{}\t❌\t[metadata]\t{}", i + 1, "ERROR: 1st object should be a CityJSON object, see https://www.cityjson.org/cityjsonseq/");
                finalresult = -1;
                break;
            }
            let re = fetch_extensions(&mut val, &extpaths);
            match re {
                Ok(_) => {
                    let valsumm = val.validate();
                    let status = get_status(&valsumm);
                    match status {
                        1 => {
                            if val.is_empty_cityjson() == false {
                                println!("{}\t❌\t[metadata]\t{}", i + 1, "ERROR: 1st object should be an CityJSON object with empty \"CityObjects\" and \"vertices\", see https://www.cityjson.org/cityjsonseq/");
                                finalresult = -1;
                                break;
                            }
                            if verbose {
                                println!(
                                    "{}\t✅\t[metadata]\t{}",
                                    i + 1,
                                    get_errors_string(&valsumm)
                                );
                            }
                        }
                        0 => {
                            finalresult = 0;
                            if !verbose {
                                println!("{}\t🟡", i + 1);
                            } else {
                                println!(
                                    "{}\t🟡\t[metadata]\t{}",
                                    i + 1,
                                    get_errors_string(&valsumm)
                                );
                            }
                        }
                        -1 => {
                            finalresult = -1;
                            if !verbose {
                                println!("{}\t❌", i + 1);
                            } else {
                                println!(
                                    "{}\t❌\t[metadata]\t{}",
                                    i + 1,
                                    get_errors_string(&valsumm)
                                );
                            }
                        }
                        _ => (),
                    }
                }
                Err(e) => {
                    finalresult = -1;
                    if !verbose {
                        println!("{}\t❌", i + 1);
                    } else {
                        let mut s = String::from("");
                        for (_ext, s2) in &e {
                            s = s + " | " + s2;
                        }
                        println!("{}\t❌\t[metadata]\t{}", i + 1, s);
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
                        1 => {
                            if verbose {
                                println!("{}\t✅\t[{}]", i + 1, val.get_cjseq_feature_id());
                            }
                        }
                        0 => {
                            if finalresult == 1 {
                                finalresult = 0;
                            }
                            println!(
                                "{}\t🟡\t[{}]\t{}",
                                i + 1,
                                val.get_cjseq_feature_id(),
                                get_errors_string(&valsumm)
                            );
                        }
                        -1 => {
                            finalresult = -1;
                            println!(
                                "{}\t❌\t[{}]\t{}",
                                i + 1,
                                val.get_cjseq_feature_id(),
                                get_errors_string(&valsumm)
                            );
                        }
                        _ => (),
                    }
                }
                Err(e) => {
                    finalresult = -1;
                    println!(
                        "{}\t❌\t[{}]\t{}",
                        i + 1,
                        val.get_cjseq_feature_id(),
                        format!("Invalid JSON object: {:?}", e)
                    );
                }
            }
        }
    }
    println!("\n");
    println!("============= SUMMARY =============");
    println!("Total lines: {:?}", linetotal);
    if finalresult == -1 {
        println!("❌ CityJSONSeq has invalid objects");
    } else if finalresult == 0 {
        println!("🟡  CityJSONSeq is valid but has warnings");
    } else {
        println!("✅ CityJSONSeq is valid");
    }
    println!("===================================");
}

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

    //-- perform validation
    let valsumm = val.validate();
    let mut has_errors = false;
    let mut has_warnings = false;

    for (criterion, summ) in valsumm.iter() {
        if verbose {
            println!(
                "{} {} {} ",
                Style::new().bold().paint("==="),
                Style::new().bold().paint(criterion),
                Style::new().bold().paint("===")
            );
            println!("{}", summ);
        }
        if summ.has_errors() == true {
            if summ.is_warning() == true {
                has_warnings = true;
            } else {
                has_errors = true;
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
    //-- Extensions
    // if val.get_input_cityjson_version() == 10 && verbose {
    // TODO: extension and v1.0?
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

fn get_errors_string(valsumm: &IndexMap<String, ValSummary>) -> String {
    let mut s = String::new();
    for (_criterion, summ) in valsumm.iter() {
        if summ.has_errors() == true {
            write!(&mut s, "{} | ", summ).expect("Problem writing String");
        }
    }
    s
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
