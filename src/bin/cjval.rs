use ansi_term::Style;
use cjval::CJValidator;

#[macro_use]
extern crate clap;

use std::path::Path;
use url::Url;

use clap::{App, AppSettings, Arg};

use anyhow::{anyhow, Result};

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

    //-- Get the validator
    let mut val = CJValidator::from_str(&s1);

    //-- Extensions
    println!("{}", Style::new().bold().paint("=== Extensions ==="));
    if val.get_input_cityjson_version() >= 11 {
        //-- if argument "-e" is passed then do not download
        if let Some(efiles) = matches.values_of("PATH") {
            let l: Vec<&str> = efiles.collect();
            let is_valid = true;
            for s in l {
                let s2 = std::fs::read_to_string(s).expect("Couldn't read Extension file");
                let scanon = Path::new(s).canonicalize().unwrap();
                let re = val.add_one_extension_from_str(&s2);
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
            } else {
                println!("\t- NONE");
            }
        }
    }
    if val.get_input_cityjson_version() == 10 {
        println!("(validation of Extensions is not supported in v1.0, upgrade to v1.1)");
    }

    let valsumm = val.validate();

    let mut has_errors = false;
    let mut has_warnings = false;
    for (criterion, summ) in valsumm.iter() {
        println!("=== {} ===", criterion);
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
