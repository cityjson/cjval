use cjval::CJValidator;
use cjval::ValSummary;
use indexmap::IndexMap;

extern crate clap;

use std::collections::HashMap;
use std::io::{self, BufRead};
use std::path::PathBuf;
use url::Url;

use clap::Parser;

use anyhow::{anyhow, Result};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};

#[derive(Parser)]
#[command(
    about = "Schema-validation of CityJSON/Seq datasets",
    override_usage = "'cjval myfile.city.json' OR 'cat mystream.city.jsonl | cjval'",
    version,
    long_about = None
)]
struct Cli {
    /// CityJSON input file
    inputfile: Option<PathBuf>,
    /// Read the CityJSON Extensions files locally instead of downloading them.
    /// More than one can be given.
    #[arg(short, long)]
    extensionfiles: Vec<PathBuf>,
}

struct ValidationResult {
    file_path: String,
    schema_version: String,
    extensions: Vec<(String, String)>,
    errors: Vec<(String, Vec<String>)>,
    warnings: Vec<(String, Vec<String>)>,
    validity: Validity,
}

#[derive(Clone, Copy, PartialEq)]
enum Validity {
    Valid,
    ValidWithWarnings,
    Invalid,
}

fn main() {
    let cli = Cli::parse();

    match cli.inputfile {
        Some(ifile) => {
            if !ifile.exists() {
                eprintln!("ERROR: Input file {} doesn't exist", ifile.display());
                std::process::exit(1);
            }
            let fext = ifile.extension().unwrap().to_str().unwrap();
            match fext {
                "json" | "JSON" => {
                    let result = validate_cityjson_file(&ifile, &cli.extensionfiles);
                    match result {
                        Ok(vr) => {
                            if let Err(e) = run_tui(vr) {
                                eprintln!("TUI error: {}", e);
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("Validation error: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                _ => {
                    eprintln!("ERROR: file extension .{} not supported (only .json)", fext);
                    std::process::exit(1);
                }
            }
        }
        None => {
            process_cjseq_stream(&cli.extensionfiles);
        }
    }
}

fn validate_cityjson_file(ifile: &PathBuf, extpaths: &Vec<PathBuf>) -> Result<ValidationResult> {
    let p1 = ifile.canonicalize()?;
    let s1 = std::fs::read_to_string(&p1)?;

    let mut val = CJValidator::from_str(&s1);

    let schema_version = if val.get_input_cityjson_version() == 0 {
        "none".to_string()
    } else {
        format!("v{}", val.get_cityjson_schema_version())
    };

    let mut extensions: Vec<(String, String)> = Vec::new();
    let mut ext_errors: Vec<String> = Vec::new();

    let re = fetch_extensions(&mut val, extpaths);
    match re {
        Ok(x) => {
            for (ext, s) in x {
                extensions.push((ext, s));
            }
        }
        Err(x) => {
            for (ext, s) in x {
                if s != "ok" {
                    ext_errors.push(format!("{}: {}", ext, s));
                }
            }
        }
    }

    let valsumm = val.validate();

    let mut errors: Vec<(String, Vec<String>)> = Vec::new();
    let mut warnings: Vec<(String, Vec<String>)> = Vec::new();

    if !ext_errors.is_empty() {
        errors.push(("Extensions".to_string(), ext_errors));
    }

    for (criterion, summ) in valsumm.iter() {
        if summ.has_errors() {
            let err_list: Vec<String> = summ.get_errors().clone();
            if summ.is_warning() {
                warnings.push((criterion.clone(), err_list));
            } else {
                errors.push((criterion.clone(), err_list));
            }
        }
    }

    let validity = if !errors.is_empty() {
        Validity::Invalid
    } else if !warnings.is_empty() {
        Validity::ValidWithWarnings
    } else {
        Validity::Valid
    };

    Ok(ValidationResult {
        file_path: p1.to_string_lossy().to_string(),
        schema_version,
        extensions,
        errors,
        warnings,
        validity,
    })
}

fn run_tui(result: ValidationResult) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui(f, &result))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    // Print final summary to stdout after TUI closes
    match result.validity {
        Validity::Valid => {
            println!("✅ File is valid");
            std::process::exit(0);
        }
        Validity::ValidWithWarnings => {
            println!("🟡 File is valid but has warnings");
            std::process::exit(0);
        }
        Validity::Invalid => {
            println!("❌ File is invalid");
            std::process::exit(1);
        }
    }
}

fn ui(frame: &mut Frame, result: &ValidationResult) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(10), // Summary
            Constraint::Min(10),    // Errors/Warnings
            Constraint::Length(1),  // Footer
        ])
        .split(frame.area());

    // Header
    let header = Paragraph::new(Line::from(vec![Span::styled(
        &result.file_path,
        Style::default().fg(Color::White),
    )]))
    .block(Block::default().borders(Borders::ALL).title("cjval"));
    frame.render_widget(header, main_layout[0]);

    // Summary panel
    render_summary(frame, main_layout[1], result);

    // Errors and Warnings panels
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[2]);

    render_errors_panel(frame, content_layout[0], result);
    render_warnings_panel(frame, content_layout[1], result);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Press ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(", ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(", or ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to exit", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(footer, main_layout[3]);
}

fn render_summary(frame: &mut Frame, area: Rect, result: &ValidationResult) {
    let (status_text, status_color) = match result.validity {
        Validity::Valid => ("✅ VALID", Color::Green),
        Validity::ValidWithWarnings => ("🟡 VALID (with warnings)", Color::Yellow),
        Validity::Invalid => ("❌ INVALID", Color::Red),
    };

    let error_count: usize = result.errors.iter().map(|(_, v)| v.len()).sum();
    let warning_count: usize = result.warnings.iter().map(|(_, v)| v.len()).sum();

    let mut summary_text = vec![
        Line::from(vec![
            Span::styled(
                "Status:     ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                status_text,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Schema:     ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(&result.schema_version),
        ]),
    ];

    if result.extensions.is_empty() {
        summary_text.push(Line::from(vec![
            Span::styled(
                "Extensions: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("none"),
        ]));
    } else {
        summary_text.push(Line::from(Span::styled(
            "Extensions:",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for (ext, _) in &result.extensions {
            let ext_name = ext.rsplit('/').next().unwrap_or(ext);
            let display = if ext.contains('/') {
                format!("  • .../{}", ext_name)
            } else {
                format!("  • {}", ext_name)
            };
            summary_text.push(Line::from(Span::raw(display)));
        }
    }

    summary_text.push(Line::from(vec![
        Span::styled(
            "Errors:     ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            error_count.to_string(),
            Style::default().fg(if error_count > 0 {
                Color::Red
            } else {
                Color::Green
            }),
        ),
    ]));
    summary_text.push(Line::from(vec![
        Span::styled(
            "Warnings:   ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            warning_count.to_string(),
            Style::default().fg(if warning_count > 0 {
                Color::Yellow
            } else {
                Color::Green
            }),
        ),
    ]));

    let summary = Paragraph::new(summary_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Summary")
                .border_style(Style::default().fg(status_color)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(summary, area);
}

fn render_errors_panel(frame: &mut Frame, area: Rect, result: &ValidationResult) {
    let mut items: Vec<ListItem> = Vec::new();
    let max_lines = area.height.saturating_sub(2) as usize; // Account for borders

    if result.errors.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No errors",
            Style::default().fg(Color::Green).italic(),
        ))));
    } else {
        let total_errors: usize = result.errors.iter().map(|(_, v)| v.len()).sum();
        let mut shown_errors = 0;
        let mut stopped = false;

        'outer: for (category, errs) in &result.errors {
            if items.len() >= max_lines.saturating_sub(1) {
                stopped = true;
                break;
            }
            items.push(ListItem::new(Line::from(Span::styled(
                format!("── {} ──", category),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))));
            for err in errs {
                if items.len() >= max_lines.saturating_sub(1) {
                    stopped = true;
                    break 'outer;
                }
                // Wrap long error messages
                let wrapped = textwrap::wrap(err, (area.width as usize).saturating_sub(4));
                for (i, line) in wrapped.iter().enumerate() {
                    if items.len() >= max_lines.saturating_sub(1) {
                        stopped = true;
                        break 'outer;
                    }
                    let prefix = if i == 0 { "• " } else { "  " };
                    items.push(ListItem::new(Line::from(Span::raw(format!(
                        "{}{}",
                        prefix, line
                    )))));
                }
                shown_errors += 1;
            }
        }

        if stopped && shown_errors < total_errors {
            let remaining = total_errors - shown_errors;
            items.push(ListItem::new(Line::from(Span::styled(
                format!(
                    "... ({} more error{})",
                    remaining,
                    if remaining == 1 { "" } else { "s" }
                ),
                Style::default().fg(Color::Red).italic(),
            ))));
        }
    }

    let errors_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "Errors",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))
            .border_style(Style::default().fg(Color::Red)),
    );

    frame.render_widget(errors_list, area);
}

fn render_warnings_panel(frame: &mut Frame, area: Rect, result: &ValidationResult) {
    let mut items: Vec<ListItem> = Vec::new();
    let max_lines = area.height.saturating_sub(2) as usize; // Account for borders

    if result.warnings.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No warnings",
            Style::default().fg(Color::Green).italic(),
        ))));
    } else {
        let total_warnings: usize = result.warnings.iter().map(|(_, v)| v.len()).sum();
        let mut shown_warnings = 0;
        let mut stopped = false;

        'outer: for (category, warns) in &result.warnings {
            if items.len() >= max_lines.saturating_sub(1) {
                stopped = true;
                break;
            }
            items.push(ListItem::new(Line::from(Span::styled(
                format!("── {} ──", category),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))));
            for warn in warns {
                if items.len() >= max_lines.saturating_sub(1) {
                    stopped = true;
                    break 'outer;
                }
                // Wrap long warning messages
                let wrapped = textwrap::wrap(warn, (area.width as usize).saturating_sub(4));
                for (i, line) in wrapped.iter().enumerate() {
                    if items.len() >= max_lines.saturating_sub(1) {
                        stopped = true;
                        break 'outer;
                    }
                    let prefix = if i == 0 { "• " } else { "  " };
                    items.push(ListItem::new(Line::from(Span::raw(format!(
                        "{}{}",
                        prefix, line
                    )))));
                }
                shown_warnings += 1;
            }
        }

        if stopped && shown_warnings < total_warnings {
            let remaining = total_warnings - shown_warnings;
            items.push(ListItem::new(Line::from(Span::styled(
                format!(
                    "... ({} more warning{})",
                    remaining,
                    if remaining == 1 { "" } else { "s" }
                ),
                Style::default().fg(Color::Yellow).italic(),
            ))));
        }
    }

    let warnings_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "Warnings",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(warnings_list, area);
}

// Stream processing for CityJSONSeq
fn process_cjseq_stream(extpaths: &Vec<PathBuf>) {
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
            if !val.is_cityjson() {
                println!(
                    "{}\t❌\t[1st-line for metadata]\t{}",
                    i + 1,
                    "ERROR: 1st line should be a CityJSON object, see https://www.cityjson.org/cityjsonseq/"
                );
                finalresult = -1;
                break;
            }
            if !val.is_empty_cityjson() {
                println!(
                    "{}\t❌\t[1st-line for metadata]\t{}",
                    i + 1,
                    "ERROR: 1st line should be an CityJSON object with empty \"CityObjects\" and \"vertices\", see https://www.cityjson.org/cityjsonseq/"
                );
                finalresult = -1;
                break;
            }
            let re = fetch_extensions(&mut val, extpaths);
            match re {
                Ok(_) => {
                    let valsumm = val.validate();
                    let status = get_status(&valsumm);
                    match status {
                        1 => {
                            println!("{}\t✅\t[1st-line for metadata]", i + 1);
                        }
                        0 => {
                            finalresult = 0;
                            println!(
                                "{}\t🟡\t[1st-line for metadata]\t{}",
                                i + 1,
                                get_errors_string(&valsumm)
                            );
                        }
                        -1 => {
                            finalresult = -1;
                            println!(
                                "{}\t❌\t[1st-line for metadata]\t{}",
                                i + 1,
                                get_errors_string(&valsumm)
                            );
                        }
                        _ => (),
                    }
                }
                Err(e) => {
                    finalresult = -1;
                    let mut s = String::from("");
                    for (_ext, s2) in &e {
                        s = s + " | " + s2;
                    }
                    println!("{}\t❌\t[1st-line for metadata]\t{}", i + 1, s);
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
                            println!("{}\t✅\t[{}]", i + 1, val.get_cjseq_feature_id());
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
        println!("🟡 CityJSONSeq is valid but has warnings");
    } else {
        println!("✅ CityJSONSeq is valid");
    }
    println!("===================================");
}

fn fetch_extensions(
    val: &mut CJValidator,
    extpaths: &Vec<PathBuf>,
) -> Result<HashMap<String, String>, HashMap<String, String>> {
    let mut b_valid = true;
    let mut d_errors: HashMap<String, String> = HashMap::new();

    if val.get_input_cityjson_version() >= 11 {
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
                }
            }
        } else {
            let re = val.get_extensions_urls();
            if re.is_some() {
                let lexts = re.unwrap();
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
            use std::fmt::Write as fmtwrite;
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
