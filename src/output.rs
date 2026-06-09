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


