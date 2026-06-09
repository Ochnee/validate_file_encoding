mod detect;
mod git;
mod mojibake;
mod output;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser)]
#[command(
    name = "validate-encoding",
    about = "Validate file encoding and detect mojibake"
)]
struct Cli {
    path: PathBuf,

    #[arg(long, help = "Expected encoding (e.g. utf-8, iso-8859-1, windows-1252)")]
    encoding: Option<String>,

    #[arg(long, short, help = "JSON output")]
    json: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let bytes = std::fs::read(&cli.path)
        .with_context(|| format!("Failed to read {}", cli.path.display()))?;

    let path_str = cli.path.to_string_lossy().to_string();

    let is_binary = mojibake::looks_binary(&bytes);

    let detected = detect::detect(&bytes, cli.encoding.as_deref())
        .context("Encoding detection failed")?;

    let valid_encodings = detect::valid_encodings(&bytes);

    let git_info = git::get_git_info(&cli.path);

    let mut warnings = if is_binary {
        Vec::new()
    } else {
        mojibake::scan(&bytes, &detected)
    };

    if let Some(ref git) = git_info {
        if let Some(ref head_enc) = git.head_encoding {
            if *head_enc != detected.name {
                warnings.push(mojibake::Warning {
                    offset: 0,
                    length: 0,
                    warning_type: "git_encoding_change".into(),
                    message: format!(
                        "Encoding changed from {} (committed) to {} (working copy) \
                         — verify intentionally changed",
                        head_enc, detected.name,
                    ),
                    bytes: vec![],
                    suggested_char: None,
                    line: 1,
                });
            }
        }
    }

    let report = output::Report {
        path: path_str,
        size: bytes.len(),
        detected_encoding: detected.name.to_string(),
        confidence: detected.confidence,
        valid_encodings,
        is_binary,
        git_info,
        warnings,
    };

    if cli.json {
        output::print_json(&report)?;
    } else {
        output::print_text(&report);
    }

    Ok(())
}
