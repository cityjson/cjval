use ansi_term::Colour::Red;
use ansi_term::Style;
use cjval::CJValidator;

extern crate clap;

use std::path::PathBuf;
use url::Url;

use clap::Parser;

use anyhow::{anyhow, Result};

#[derive(Parser)]
#[command(version, about = "Validation of a CityJSON file", long_about = None)]
struct Cli {
    /// CityJSON input file
    inputfile: PathBuf,
    /// Read the CityJSON Extensions files locally instead of downloading them.
    /// More than one can be given.
    #[arg(short, long)]
    extensionfiles: Vec<PathBuf>,
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
    std::process::exit(0);
}

fn main() {
    let cli = Cli::parse();

    if !cli.inputfile.exists() {
        eprintln!(
            "ERROR: Input file {} doesn't exist",
            cli.inputfile.display()
        );
        std::process::exit(0);
    }

    let p1 = cli.inputfile.canonicalize().unwrap();

    let s1 = std::fs::read_to_string(&p1).expect("Couldn't read CityJSON file");
    println!(
        "{}",
        Style::new().bold().paint("=== Input CityJSON file ===")
    );
    println!("{:?}", p1);

    //-- Get the validator
    let mut val = CJValidator::from_str(&s1);

    //-- print the schema version used
    println!("{}", Style::new().bold().paint("=== CityJSON schemas ==="));
    if val.get_input_cityjson_version() == 0 {
        println!("none");
    } else {
        println!("v{} (builtin)", val.get_cityjson_schema_version());
    }

    //-- Extensions
    println!("{}", Style::new().bold().paint("=== Extensions ==="));
    if val.get_input_cityjson_version() == 10 {
        println!("(validation of Extensions is not supported in CityJSON v1.0, upgrade to v1.1)");
    }
    if val.get_input_cityjson_version() >= 11 {
        //-- if argument "-e" is passed then do not download
        if cli.extensionfiles.len() > 0 {
            for fext in cli.extensionfiles {
                if fext.exists() {
                    let fexts =
                        std::fs::read_to_string(&fext).expect("Couldn't read Extension file");
                    let re = val.add_one_extension_from_str(&fexts);
                    match re {
                        Ok(()) => println!("- {}.. ok", fext.to_str().unwrap()),
                        Err(e) => {
                            println!("- {}.. {}", fext.to_str().unwrap(), Red.paint("ERROR"));
                            println!("({})", e);
                            summary_and_bye(-1);
                        }
                    }
                } else {
                    println!("- {}.. {}", fext.to_str().unwrap(), Red.paint("ERROR"));
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
                                Ok(()) => println!("- {}.. ok", ext),
                                Err(e) => {
                                    println!("- {}.. {}", ext, Red.paint("ERROR"));
                                    println!("({})", e);
                                    summary_and_bye(-1);
                                }
                            }
                        }
                        Err(e) => {
                            println!("- {}.. {} \n\t{}", ext, e, Red.paint("ERROR"));
                            summary_and_bye(-1);
                        }
                    }
                }
            } else {
                println!("none");
            }
        }
    }

    let valsumm = val.validate();
    let mut has_errors = false;
    let mut has_warnings = false;
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

    //-- bye-bye
    if has_errors == false && has_warnings == false {
        summary_and_bye(1);
    } else if has_errors == false && has_warnings == true {
        summary_and_bye(0);
    } else {
        summary_and_bye(-1);
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
