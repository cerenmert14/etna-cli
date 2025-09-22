use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::{
    collections::{BTreeSet, HashMap},
    fmt::Display,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordType {
    /// Information about a single test case execution
    TestCase,
    /// Information about a testing campaign consisting of multiple test cases
    Campaign,
    /// Free-form log message
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Passed,
    FoundBug,
    GaveUp,
    TimedOut,
    Aborted,
    Unknown,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Status::Passed => write!(f, "passed"),
            Status::FoundBug => write!(f, "found_bug"),
            Status::GaveUp => write!(f, "gave_up"),
            Status::TimedOut => write!(f, "timed_out"),
            Status::Aborted => write!(f, "aborted"),
            Status::Unknown => write!(f, "unknown"),
        }
    }
}

fn default_record_type() -> RecordType {
    RecordType::TestCase
}

/// Describes the inputs to and result of running a test function on a particular input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseRecord {
    /// A tag which labels this observation as data about a specific test case.
    /// Always "test_case".
    #[serde(rename = "type", default = "default_record_type")]
    pub kind: RecordType,

    /// Whether the test passed, failed, or was aborted before completion.
    pub status: Status,

    /// If non-empty, the reason for which the test failed or was abandoned.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub status_reason: String,

    /// String representation of the input (e.g., "test_a(a=1)").
    #[serde(skip_serializing_if = "String::is_empty")]
    pub representation: String,

    /// Structured (JSON) representation of the input arguments.
    /// May be absent or incomplete if `status` is `gave_up`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<JsonMap<String, JsonValue>>,

    /// How the input was generated, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub how_generated: Option<String>,

    /// Runtime observations (e.g., target() scores, event() tags).
    /// Defaults to an empty object.
    #[serde(default)]
    pub features: JsonMap<String, JsonValue>,

    /// Mapping of filename -> set of covered line numbers (unique, >= 1).
    /// None if coverage info is not available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<HashMap<String, BTreeSet<u32>>>,

    /// Non-overlapping timings in seconds (e.g., "execute:test", "overall:gc").
    /// Values must be >= 0; not enforced at type level.
    #[serde(default)]
    pub timing: HashMap<String, f64>,

    /// Arbitrary metadata (e.g., traceback for failing tests).
    #[serde(default)]
    pub metadata: JsonMap<String, JsonValue>,

    /// The name or representation of the test function we're running.
    #[serde(rename = "property")]
    pub property_name: String,

    /// Unix timestamp (seconds since epoch) at which we started running this test.
    pub run_start: i64,
}

fn campaign_record_type() -> RecordType {
    RecordType::Campaign
}

/// Describes the inputs to and result of running a test function on a particular input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignRecord {
    /// A tag which labels this observation as data about a specific test case.
    /// Always "test_case".
    #[serde(rename = "type", default = "campaign_record_type")]
    pub kind: RecordType,

    /// Whether the campaign passed all the tests, failed by finding a counterexample, hit its discard limit,
    /// timed out, or was aborted before completion.
    pub status: Status,
    /// If non-empty, the reason for which the test failed or was abandoned.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub status_reason: String,

    /// The number of passed test cases in this campaign.
    /// Must be >= 0.
    pub passed: u64,

    /// The number of failed test cases in this campaign.
    /// Must be 0 if `status` is `passed`, `timed_out`, `gave_up` or `aborted`.
    /// Must be > 0 if `status` is `failed`.
    pub failed: u64,

    /// The number of test cases that were discarded in this campaign.
    /// Must be >= 0.
    pub discarded: u64,

    /// The number of timed out test cases in this campaign.
    /// Must be >= 0.
    pub timed_out: u64,

    /// The number of test cases that were aborted in this campaign.
    /// Must be >= 0.
    pub aborted: u64,

    /// String representation of the input (e.g., "test_a(a=1)").
    pub representation: String,

    /// Structured (JSON) representation of the input arguments.
    /// May be absent or incomplete if `status` is `gave_up`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<JsonMap<String, JsonValue>>,

    /// Runtime observations (e.g., target() scores, event() tags).
    /// Defaults to an empty object.
    #[serde(default)]
    pub features: JsonMap<String, JsonValue>,

    /// Non-overlapping timings in seconds (e.g., "execute:test", "overall:gc").
    /// Values must be >= 0; not enforced at type level.
    #[serde(default)]
    pub timing: HashMap<String, f64>,

    /// Arbitrary metadata (e.g., traceback for failing tests).
    #[serde(default)]
    pub metadata: JsonMap<String, JsonValue>,

    /// The name or representation of the test function we're running.
    #[serde(rename = "property")]
    pub property_name: String,

    /// Unix timestamp (seconds since epoch) at which we started running this test.
    pub run_start: i64,
}
