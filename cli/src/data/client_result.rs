use crate::data::client_result::ClientResultType::Fail;
use crate::data::workspace::Workspace;
use colored::Colorize;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::{process_id_text, process_text};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::Instant;

#[derive(Serialize, Clone, Debug)]
pub struct ClientResult {
    // All messages

    /// Error messages
    #[serde(rename = "ErrMsg")]
    err_msg: Vec<String>,

    /// Warning messages
    #[serde(rename = "WarnMsg")]
    warn_msg: Vec<String>,

    /// Normal messages
    #[serde(rename = "LogMsg")]
    log_msg: Vec<String>,

    /// Metadata
    #[serde(rename = "Metadata")]
    metadata: HashMap<String, String>,

    /// Error type
    #[serde(rename = "ResultType")]
    result_type: ClientResultType,

    /// Error type
    #[serde(rename = "ElapsedSeconds")]
    elapsed_secs: f64,

    /// Raw message processing function (raw content, remaining count) -> output content
    #[serde(skip_serializing)]
    query_process: fn(raw: String, remaining: i32) -> String,

    /// Debug output
    #[serde(skip_serializing)]
    debug: bool,

    #[serde(skip_serializing)]
    start_instant: Instant
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ClientResultType {
    /// Query
    #[serde(rename = "Query")]
    Query,

    /// Failure
    #[serde(rename = "Fail")]
    Fail,

    /// Success
    #[serde(rename = "Success")]
    Success
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct QueryResult {
    /// Query
    #[serde(rename = "Query")]
    query: Vec<String>,

    /// Metadata
    #[serde(rename = "Metadata")]
    metadata: HashMap<String, String>,
}

impl ClientResult {
    pub async fn debug_mode() -> bool {
        let workspace = Workspace::read().await;
        if let Some(client) = workspace.client {
            client.debug
        } else {
            false
        }
    }

    async fn setup() -> ClientResult {
        let debug = Self::debug_mode().await;

        ClientResult {
            err_msg: vec![],
            warn_msg: vec![],
            log_msg: vec![],
            metadata: Default::default(),
            result_type: ClientResultType::Query,
            elapsed_secs: 0.0,
            query_process : ClientResultQueryProcess::line_by_line_compressed,
            debug,
            start_instant: Instant::now()
        }
    }

    pub async fn result() -> ClientResult {
        let mut r = Self::setup().await;
        r.result_type = ClientResultType::Success;
        r
    }

    pub async fn query(process: fn(raw: String, remaining: i32) -> String) -> ClientResult {
        let mut r = Self::setup().await;
        r.query_process = process;
        r
    }

    pub fn log(&mut self, msg: &str) {
        if self.result_type != ClientResultType::Query && ! self.debug {
            println!("Log: {}", &msg);
        }
        self.log_msg.push(
            if self.debug {
                strip_ansi_escapes::strip_str(
                    process_text(msg.to_string())
                )
            } else {
                msg.to_string()
            }
        );
    }

    pub fn warn(&mut self, msg: &str) {
        if self.result_type != ClientResultType::Query && ! self.debug {
            println!("{}", format!("Warn: {}", &msg).bright_yellow());
        }
        self.warn_msg.push(
            strip_ansi_escapes::strip_str(
                process_text(msg.to_string())
            )
        );
    }

    pub fn err(&mut self, msg: &str) {
        if self.result_type != ClientResultType::Query && ! self.debug {
            println!("{}", format!("Err: {}", &msg).bright_red());
        }
        self.err_msg.push(
            strip_ansi_escapes::strip_str(
                process_text(msg.to_string())
            )
        );
        if self.result_type != ClientResultType::Query {
            self.result_type = Fail;
        }
    }

    pub fn metadata(&mut self, data_key: String, data_val: String) {
        self.metadata.insert(process_id_text(data_key), process_text(data_val));
    }

    /// The function will print a message and return a string in debug mode,
    /// but will not return anything when debug mode is turned off.
    pub fn end_print(mut self) -> String {

        // Computing elapsed time
        self.elapsed_secs = self.start_instant.elapsed().as_secs_f64();

        // Debug output, serialize directly
        if self.debug {
            if self.result_type == ClientResultType::Query {
                let mut result = serde_json::to_string(&QueryResult::from(self)).unwrap_or("{}".to_string());
                result = format!("query:{}", &result);
                result
            } else {
                let mut result = serde_json::to_string(&self).unwrap_or("{}".to_string());
                result = format!("result:{}", &result);
                result
            }
        } else {
            // Otherwise, output based on conditions
            match self.result_type {
                ClientResultType::Query => {
                    // Process all log messages
                    let logs = self.log_msg;
                    let mut result = String::new();
                    let mut remain : i32 = logs.len() as i32 - 1;
                    for log in logs {
                        result.push_str((self.query_process)(log, remain).as_str());
                        remain -= 1;
                    }
                    println!("{}", &result);
                }
                _ => {
                    let mut result = String::new();
                    result.push_str("( ");
                    if self.log_msg.len() > 0 {
                        result.push_str(format!("{} log ", self.log_msg.len()).bright_green().to_string().as_str());
                    }
                    if self.warn_msg.len() > 0 {
                        result.push_str(format!("{} warn ", self.warn_msg.len()).bright_yellow().to_string().as_str());
                    }
                    if self.err_msg.len() > 0 {
                        result.push_str(format!("{} err ", self.err_msg.len()).bright_red().to_string().as_str());
                    }
                    result.push_str(")");
                    println!("{} in {} secs.", result, format!("{:.2}", self.elapsed_secs).cyan());
                }
            }
            String::new()
        }
    }

    pub fn err_and_end(mut self, msg: &str) {
        self.err(msg);
        self.end_print();
    }

    pub fn has_result(&self) -> bool {
        self.log_msg.len() > 0 || self.warn_msg.len() > 0 || self.err_msg.len() > 0
    }

    pub fn combine(&mut self, other: ClientResult) -> Result<(), ()> {
        // Cannot combine when type is Query
        if self.result_type == ClientResultType::Query || other.result_type == ClientResultType::Query {
            return Err(());
        }

        if other.result_type == Fail {
            self.result_type = Fail;
        }

        for log in other.log_msg {
            self.log_msg.push(log);
        }

        for warn in other.warn_msg {
            self.warn_msg.push(warn);
        }

        for err in other.err_msg {
            self.err_msg.push(err);
        }

        // Metadata takes precedence from the new one
        for metadata_kvp in other.metadata {
            self.metadata.insert(metadata_kvp.0, metadata_kvp.1);
        }

        Ok(())
    }

    pub fn combine_unchecked(&mut self, other: Option<ClientResult>) {
        if let Some(other) = other {
            let _ = self.combine(other);
        }
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }
}

pub struct ClientResultQueryProcess;
impl ClientResultQueryProcess {
    pub fn line_by_line_compressed (raw: String, remaining: i32) -> String {
        if remaining == 0 {
            format!("{}", process_text(raw.to_string()))
        } else {
            format!("{}\n", process_text(raw.to_string()))
        }
    }

    pub fn line_by_line (raw: String, remaining: i32) -> String {
        if remaining == 0 {
            format!("{}", raw.to_string())
        } else {
            format!("{}\n", raw.to_string())
        }
    }

    pub fn direct (raw: String, _remaining: i32) -> String {
        raw
    }

    pub fn comma (raw: String, remaining: i32) -> String {
        if remaining == 0 {
            format!("{}", process_text(raw.to_string()))
        } else {
            format!("{}, ", process_text(raw.to_string()))
        }
    }

    pub fn comma_quotation_marks (raw: String, remaining: i32) -> String {
        if remaining == 0 {
            format!("\"{}\"", process_text(raw.to_string()))
        } else {
            format!("\"{}\", ", process_text(raw.to_string()))
        }
    }
}

impl From<ClientResult> for QueryResult {
    fn from(value: ClientResult) -> Self {
        Self {
            query: value.log_msg,
            metadata: value.metadata,
        }
    }
}