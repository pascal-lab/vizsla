use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeLensData {
    pub version: i32,
    pub kind: CodeLensDataKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CodeLensDataKind {
    Instantiation(lsp_types::TextDocumentPositionParams),
}
