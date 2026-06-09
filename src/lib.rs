mod detect;
mod git;
mod mojibake;
mod output;

use std::path::Path;

use mcp_plugin_sdk::tool_plugin;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct ValidateFileArgs {
    /// Path to the file to validate
    file_path: String,
    /// Expected encoding (e.g. utf-8, iso-8859-1, windows-1252)
    #[schemars(default)]
    encoding: Option<String>,
}

fn run_validation(path: &str, encoding: Option<&str>) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;

    let is_binary = mojibake::looks_binary(&bytes);

    let detected =
        detect::detect(&bytes, encoding).map_err(|e| format!("Detection failed: {}", e))?;

    let valid_encodings = detect::valid_encodings(&bytes);

    let git_info = git::get_git_info(Path::new(path));

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
        path: path.to_string(),
        size: bytes.len(),
        detected_encoding: detected.name.to_string(),
        confidence: detected.confidence,
        valid_encodings,
        is_binary,
        git_info,
        warnings,
    };

    serde_json::to_string_pretty(&report)
        .map_err(|e| format!("Failed to serialize report: {}", e))
}

#[tool_plugin]
mod tools {
    use super::ValidateFileArgs;

    /// Validate file encoding and detect mojibake (garbled text) in Nordic/European text files
    #[tool]
    fn validate_file(args: ValidateFileArgs) -> Result<String, String> {
        super::run_validation(&args.file_path, args.encoding.as_deref())
    }
}
