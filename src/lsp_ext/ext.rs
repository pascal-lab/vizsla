use lspt::notification::Notification;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeLensData {
    pub version: i32,
    pub kind: CodeLensDataKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CodeLensDataKind {
    Instantiation(TextDocumentPositionParams),
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentPositionParams {
    pub text_document: lspt::TextDocumentIdentifier,
    pub position: lspt::Position,
}

pub(crate) mod sema_token_types {
    pub(crate) const COMMENT: &str = "comment";
    pub(crate) const DECORATOR: &str = "decorator";
    pub(crate) const ENUM_MEMBER: &str = "enumMember";
    pub(crate) const ENUM: &str = "enum";
    pub(crate) const FUNCTION: &str = "function";
    pub(crate) const INTERFACE: &str = "interface";
    pub(crate) const KEYWORD: &str = "keyword";
    pub(crate) const MACRO: &str = "macro";
    pub(crate) const METHOD: &str = "method";
    pub(crate) const NAMESPACE: &str = "namespace";
    pub(crate) const NUMBER: &str = "number";
    pub(crate) const OPERATOR: &str = "operator";
    pub(crate) const PARAMETER: &str = "parameter";
    pub(crate) const PROPERTY: &str = "property";
    pub(crate) const STRING: &str = "string";
    pub(crate) const STRUCT: &str = "struct";
    pub(crate) const TYPE_PARAMETER: &str = "typeParameter";
    pub(crate) const VARIABLE: &str = "variable";
    pub(crate) const TYPE: &str = "type";

    pub(crate) const CLK_PORT: &str = "port_clock";
    pub(crate) const RST_PORT: &str = "port_reset";
    pub(crate) const OTHERS_PORT: &str = "port_generic";
    pub(crate) const INSTANCE: &str = "instance";
    pub(crate) const TYPE_ALIAS: &str = "type_alias";
    pub(crate) const GENERIC: &str = "generic";

    pub(crate) fn fallback(token: &'static str) -> Option<&'static str> {
        match token {
            CLK_PORT => Some(KEYWORD),
            RST_PORT => Some(PROPERTY),
            OTHERS_PORT => Some(PARAMETER),
            INSTANCE => Some(VARIABLE),
            TYPE_ALIAS => Some(TYPE),
            GENERIC => Some(TYPE_PARAMETER),
            _ => Some(token),
        }
    }
}

pub(crate) const SEMA_TOKENS_TYPES: &[&str] = &[
    sema_token_types::COMMENT,
    sema_token_types::DECORATOR,
    sema_token_types::ENUM_MEMBER,
    sema_token_types::ENUM,
    sema_token_types::FUNCTION,
    sema_token_types::INTERFACE,
    sema_token_types::KEYWORD,
    sema_token_types::MACRO,
    sema_token_types::METHOD,
    sema_token_types::NAMESPACE,
    sema_token_types::NUMBER,
    sema_token_types::OPERATOR,
    sema_token_types::PARAMETER,
    sema_token_types::PROPERTY,
    sema_token_types::STRING,
    sema_token_types::STRUCT,
    sema_token_types::TYPE_PARAMETER,
    sema_token_types::VARIABLE,
    sema_token_types::TYPE,
    sema_token_types::CLK_PORT,
    sema_token_types::RST_PORT,
    sema_token_types::OTHERS_PORT,
    sema_token_types::INSTANCE,
    sema_token_types::TYPE_ALIAS,
    sema_token_types::GENERIC,
];
#[derive(Default)]
pub(crate) struct SemaTokenModifierSet(pub(crate) u32);

impl SemaTokenModifierSet {
    pub(crate) fn finish(self) -> u32 {
        self.0
    }
}

pub(crate) mod sema_token_modifiers {
    pub(crate) const DECLARATION: &str = "declaration";
    pub(crate) const DEFINITION: &str = "definition";
    pub(crate) const READONLY: &str = "readonly";
    pub(crate) const STATIC: &str = "static";
    pub(crate) const DEPRECATED: &str = "deprecated";
    pub(crate) const ABSTRACT: &str = "abstract";
    pub(crate) const ASYNC: &str = "async";
    pub(crate) const MODIFICATION: &str = "modification";
    pub(crate) const DOCUMENTATION: &str = "documentation";
    pub(crate) const DEFAULT_LIBRARY: &str = "defaultLibrary";

    pub(crate) const READ: &str = "read";
    pub(crate) const WRITE: &str = "write";
    pub(crate) const REF: &str = "ref";
    pub(crate) const DEF: &str = "definition";

    pub(crate) fn fallback(token: &'static str) -> Option<&'static str> {
        match token {
            REF => Some(MODIFICATION),
            _ => Some(token),
        }
    }
}

pub(crate) const SEMA_TOKENS_MODIFIERS: &[&str] = &[
    sema_token_modifiers::DECLARATION,
    sema_token_modifiers::DEFINITION,
    sema_token_modifiers::READONLY,
    sema_token_modifiers::STATIC,
    sema_token_modifiers::DEPRECATED,
    sema_token_modifiers::ABSTRACT,
    sema_token_modifiers::ASYNC,
    sema_token_modifiers::MODIFICATION,
    sema_token_modifiers::DOCUMENTATION,
    sema_token_modifiers::DEFAULT_LIBRARY,
    sema_token_modifiers::READ,
    sema_token_modifiers::WRITE,
    sema_token_modifiers::REF,
    sema_token_modifiers::DEF,
];

impl std::ops::BitOrAssign<&'static str> for SemaTokenModifierSet {
    fn bitor_assign(&mut self, rhs: &'static str) {
        let Some(idx) = SEMA_TOKENS_MODIFIERS.iter().position(|it| it == &rhs) else {
            tracing::debug!(?rhs, "unknown semantic token modifier");
            return;
        };
        self.0 |= 1 << idx;
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionData {
    pub code_action_params: lspt::CodeActionParams,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
}

#[derive(Debug, Error)]
pub enum CodeActionResolveError {
    #[error("code action without data")]
    NoData,
    #[error("stale code action")]
    Stable,
    #[error("invalid action id: {0}")]
    InvalidId(String),
}

pub const RUN_QIHE_ANALYSIS_COMMAND: &str = "vizsla.server.runQiheAnalysis";
pub const RELOAD_WORKSPACE_COMMAND: &str = "vizsla.server.reloadWorkspace";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunQiheAnalysisParams {
    pub uri: lspt::Uri,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QiheStatusParams {
    pub token: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub enum QiheStatusNotification {}

impl Notification for QiheStatusNotification {
    type Params = QiheStatusParams;

    const METHOD: &'static str = "vizsla/qiheStatus";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QiheLogParams {
    pub token: String,
    pub message: String,
}

pub enum QiheLogNotification {}

impl Notification for QiheLogNotification {
    type Params = QiheLogParams;

    const METHOD: &'static str = "vizsla/qiheLog";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectStatusState {
    Loading,
    Loaded,
    #[serde(rename = "none")]
    NoManifest,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStatusParams {
    pub state: ProjectStatusState,
    pub manifest_uris: Vec<lspt::Uri>,
    pub unconfigured_root_uris: Vec<lspt::Uri>,
    pub workspace_count: usize,
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub enum ProjectStatusNotification {}

impl Notification for ProjectStatusNotification {
    type Params = ProjectStatusParams;

    const METHOD: &'static str = "vizsla/projectStatus";
}
