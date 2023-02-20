#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::time::SystemTime;

pub mod copy;
pub mod proxy;
pub mod serialize;
pub mod write;

#[derive(Clone, Copy)]
pub enum Severity {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl From<u8> for Severity {
    fn from(num: u8) -> Self {
        match num {
            1..=4 => Self::Trace,
            5..=8 => Self::Debug,
            9..=12 => Self::Info,
            13..=16 => Self::Warn,
            17..=20 => Self::Error,
            21..=24 => Self::Fatal,
            _ => Self::Fatal,
        }
    }
}

impl From<Severity> for u8 {
    fn from(s: Severity) -> Self {
        match s {
            Severity::Trace => 1,
            Severity::Debug => 5,
            Severity::Info => 9,
            Severity::Warn => 13,
            Severity::Error => 17,
            Severity::Fatal => 21,
        }
    }
}

impl Severity {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
            Self::Fatal => "fatal",
        }
    }
}

pub struct Item {
    pub timestamp: SystemTime,
    pub severity: Severity,
    pub body: String,
    pub attributes: BTreeMap<String, String>,
    pub resource: BTreeMap<String, String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
}

impl Item {
    pub fn new(body: &str, attr: BTreeMap<String, String>) -> Self {
        Self {
            timestamp: SystemTime::now(),
            severity: Severity::Trace,
            body: body.into(),
            attributes: attr,
            resource: BTreeMap::new(),
            trace_id: None,
            span_id: None,
        }
    }

    pub fn with_resource_keys(self, keys: &[&str]) -> Self {
        let resource = BTreeMap::from_iter(keys.iter().map(|&key: &&str| (key.into(), "".into())));
        Self {
            timestamp: self.timestamp,
            severity: self.severity,
            body: self.body,
            attributes: self.attributes,
            resource,
            trace_id: self.trace_id,
            span_id: self.span_id,
        }
    }
}
