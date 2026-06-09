use serde::Serialize;

use crate::git::GitInfo;
use crate::mojibake::Warning;

#[derive(Serialize)]
pub struct Report {
    pub path: String,
    pub size: usize,
    pub detected_encoding: String,
    pub confidence: f32,
    pub valid_encodings: Vec<&'static str>,
    pub is_binary: bool,
    pub git_info: Option<GitInfo>,
    pub warnings: Vec<Warning>,
}

impl Serialize for GitInfo {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("GitInfo", 4)?;
        st.serialize_field("tracked", &self.tracked)?;
        st.serialize_field("repo_root", &self.repo_root)?;
        st.serialize_field("head_encoding", &self.head_encoding)?;
        st.serialize_field("head_confidence", &self.head_confidence)?;
        st.end()
    }
}

impl Serialize for Warning {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("Warning", 7)?;
        st.serialize_field("offset", &self.offset)?;
        st.serialize_field("length", &self.length)?;
        st.serialize_field("warning_type", &self.warning_type)?;
        st.serialize_field("message", &self.message)?;
        st.serialize_field("bytes", &self.bytes)?;
        st.serialize_field("suggested_char", &self.suggested_char)?;
        st.serialize_field("line", &self.line)?;
        st.end()
    }
}

pub fn print_text(report: &Report) {
    println!("File:          {}", report.path);
    println!("Size:          {} bytes", report.size);

    if report.is_binary {
        println!("{}", ansi("⚠  Binary file — skipping text analysis", 33));
        println!("Valid as:      binary");
        return;
    }

    println!(
        "Detected:      {} (confidence: {:.0}%)",
        report.detected_encoding,
        report.confidence * 100.0
    );

    println!(
        "Valid as:      {}",
        report.valid_encodings.join(", ")
    );

    if let Some(ref git) = report.git_info {
        if git.tracked {
            if let Some(ref head_enc) = git.head_encoding {
                println!(
                    "{}",
                    ansi(
                        &format!(
                            "Git:           tracked — committed as {}, currently {}",
                            head_enc, report.detected_encoding
                        ),
                        36,
                    )
                );
            } else {
                println!("Git:           tracked in repo");
            }
        } else {
            println!("Git:           untracked");
        }
    }

    if report.warnings.is_empty() {
        println!("{}", ansi("✓ No issues detected", 32));
    } else {
        let errors = report
            .warnings
            .iter()
            .filter(|w| w.warning_type == "replacement_char")
            .count();
        let mojibake_count = report
            .warnings
            .iter()
            .filter(|w| w.warning_type != "replacement_char" && w.warning_type != "git_encoding_change")
            .count();
        let git_changes = report
            .warnings
            .iter()
            .filter(|w| w.warning_type == "git_encoding_change")
            .count();

        println!("Errors:        {}", errors);
        println!("Mojibake:      {} warnings", mojibake_count);
        if git_changes > 0 {
            println!("Git changes:   {}", git_changes);
        }

        for w in &report.warnings {
            let (label, color) = match w.warning_type.as_str() {
                "replacement_char" => ("REPLACEMENT", 31),
                "utf8_as_latin1" => ("UTF8→LATIN1", 33),
                "latin1_as_utf8" => ("LATIN1→UTF8", 33),
                "git_encoding_change" => ("GIT ENCODING", 36),
                _ => ("WARNING", 33),
            };

            println!(
                "  {} {}",
                ansi(&format!("[{:>13}]", label), color),
                ansi(&w.message, 0)
            );

            if let Some(ref suggested) = w.suggested_char {
                println!(
                    "             {}",
                    ansi(&format!("↳ should be: \"{}\"", suggested), 36)
                );
            }
        }
    }
}

pub fn print_json(report: &Report) -> Result<(), anyhow::Error> {
    println!("{}", serde_json::to_string_pretty(report)?);
    Ok(())
}

fn ansi(text: &str, code: u8) -> String {
    format!("\x1b[{}m{}\x1b[0m", code, text)
}
