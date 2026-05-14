#![allow(dead_code)]

use bitcoin::{
    Sequence, Txid,
    absolute::{Height, LockTime, Time},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionSourceMode {
    Symbolic,
    Raw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Confirmed,
    Unconfirmed,
    Candidate,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthoredLockTime {
    Raw(u32),
    Height(Height),
    Time(Time),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NormalizedLockTime {
    pub authored: Option<AuthoredLockTime>,
    pub consensus: Option<LockTime>,
}

impl NormalizedLockTime {
    pub fn from_consensus(locktime: LockTime) -> Self {
        Self {
            authored: Some(match locktime {
                LockTime::Blocks(height) => AuthoredLockTime::Height(height),
                LockTime::Seconds(time) => AuthoredLockTime::Time(time),
            }),
            consensus: Some(locktime),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceKind {
    Exact,
    RelativeHeight,
    RelativeTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthoredSequence {
    Raw(u32),
    RelativeHeight(u32),
    RelativeTime(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalizedSequence {
    pub authored: Option<AuthoredSequence>,
    pub consensus: Option<Sequence>,
    pub kind: Option<SequenceKind>,
}

impl Default for NormalizedSequence {
    fn default() -> Self {
        Self {
            authored: None,
            consensus: None,
            kind: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmountDisplay {
    Sat,
    Expr,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    Spend,
    CandidateSpend,
    DoubleSpendCompetitor,
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Validity {
    Valid,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionStatus {
    Satisfied,
    Unsatisfied,
    Candidate,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emphasis {
    None,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookmarkTargetKind {
    Transaction,
    Input,
    Output,
    Edge,
    View,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutEngine {
    Auto,
    Graphviz,
    Dagre,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedDocumentMeta {
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedCondition {
    pub label: String,
    pub status: ConditionStatus,
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NormalizedAnnotations {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tooltip: Option<String>,
    pub conditions: Vec<NormalizedCondition>,
    pub hidden_details: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedInlineStyle {
    pub stroke: Option<String>,
    pub fill: Option<String>,
    pub text: Option<String>,
    pub emphasis: Emphasis,
}

impl Default for NormalizedInlineStyle {
    fn default() -> Self {
        Self {
            stroke: None,
            fill: None,
            text: None,
            emphasis: Emphasis::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizedSpendRef {
    None,
    Symbolic {
        raw: String,
        tx_ref: String,
        vout: u32,
    },
    Txid {
        raw: String,
        txid_hex: String,
        vout: u32,
    },
    Invalid {
        raw: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedPrevout {
    Resolved {
        out_uid: String,
        tx_uid: String,
        vout: u32,
    },
    Dangling,
    Ambiguous,
    Coinbase,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedInput {
    pub in_uid: String,
    pub vin: u32,
    pub spends_ref: NormalizedSpendRef,
    pub resolved_prevout: ResolvedPrevout,
    pub sequence: NormalizedSequence,
    pub annotations: NormalizedAnnotations,
    pub classes: Vec<String>,
    pub style: Option<NormalizedInlineStyle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedAmount {
    pub amount_sat: Option<u64>,
    pub amount_expr: Option<String>,
    pub display_primary: AmountDisplay,
    pub display_secondary: AmountDisplay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedOutput {
    pub out_uid: String,
    pub vout: u32,
    pub amount: NormalizedAmount,
    pub descriptor: Option<String>,
    pub is_spent: bool,
    pub spent_by: Vec<String>,
    pub annotations: NormalizedAnnotations,
    pub classes: Vec<String>,
    pub style: Option<NormalizedInlineStyle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionSource {
    pub raw_hex: Option<String>,
    pub author_index: u32,
    pub author_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedTransaction {
    pub tx_uid: String,
    pub symbolic_id: Option<String>,
    pub txid: Option<Txid>,
    pub source_mode: TransactionSourceMode,
    pub status: TransactionStatus,
    pub version: Option<u32>,
    pub locktime: NormalizedLockTime,
    pub inputs: Vec<NormalizedInput>,
    pub outputs: Vec<NormalizedOutput>,
    pub annotations: NormalizedAnnotations,
    pub classes: Vec<String>,
    pub style: Option<NormalizedInlineStyle>,
    pub source: TransactionSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedSpendEdge {
    pub edge_uid: String,
    pub from_out_uid: String,
    pub to_in_uid: String,
    pub kind: EdgeKind,
    pub validity: Validity,
    pub annotations: NormalizedAnnotations,
    pub descriptor: Option<String>,
    pub classes: Vec<String>,
    pub style: Option<NormalizedInlineStyle>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedRank {
    pub rank_id: String,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedLayout {
    pub engine: LayoutEngine,
    pub direction: LayoutDirection,
    pub ranks: Vec<NormalizedRank>,
    pub positions: Vec<(String, NormalizedPosition)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedClassStyle {
    pub transaction: Option<NormalizedInlineStyle>,
    pub input: Option<NormalizedInlineStyle>,
    pub output: Option<NormalizedInlineStyle>,
    pub edge: Option<NormalizedInlineStyle>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NormalizedStyles {
    pub classes: Vec<(String, NormalizedClassStyle)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedBookmarkTarget {
    pub kind: BookmarkTargetKind,
    pub uid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedBookmark {
    pub bookmark_id: String,
    pub label: String,
    pub target: NormalizedBookmarkTarget,
    pub order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub path: String,
    pub related_uids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedGraph {
    pub format_version: String,
    pub document: NormalizedDocumentMeta,
    pub transactions: Vec<NormalizedTransaction>,
    pub edges: Vec<NormalizedSpendEdge>,
    pub bookmarks: Vec<NormalizedBookmark>,
    pub layout: NormalizedLayout,
    pub styles: NormalizedStyles,
    pub diagnostics: Vec<NormalizedDiagnostic>,
}

impl Default for NormalizedLayout {
    fn default() -> Self {
        Self {
            engine: LayoutEngine::Auto,
            direction: LayoutDirection::LeftToRight,
            ranks: Vec::new(),
            positions: Vec::new(),
        }
    }
}

impl Default for NormalizedGraph {
    fn default() -> Self {
        Self {
            format_version: "0.1".to_owned(),
            document: NormalizedDocumentMeta {
                id: None,
                title: None,
                description: None,
            },
            transactions: Vec::new(),
            edges: Vec::new(),
            bookmarks: Vec::new(),
            layout: NormalizedLayout::default(),
            styles: NormalizedStyles::default(),
            diagnostics: Vec::new(),
        }
    }
}
