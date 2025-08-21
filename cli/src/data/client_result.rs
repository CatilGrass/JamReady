use crate::data::workspace::Workspace;
use colored::Colorize;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::{process_id_text, process_text};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::data::client_result::ClientResultType::Fail;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ClientResult {
    // All messages

    /// Error messages
    #[serde(rename = "ErrMsg")]
    err_msg: Vec<String>,

    /// Warning messages
    #[serde(rename = "WarnMsg")]
    warn_msg: Vec<String>,

    /// Information messages
    #[serde(rename = "InfoMsg")]
    info_msg: Vec<String>,

    /// Metadata
    #[serde(rename = "Metadata")]
    metadata: HashMap<String, String>,

    /// Error type
    #[serde(rename = "ResultType")]
    result_type: ClientResultType,

    /// Raw message processing function (raw content, remaining count) -> output content
    #[serde(skip_serializing)]
    query_process: fn(raw: String, remaining: i32) -> String,

    /// Debug output
    #[serde(skip_serializing)]
    debug: bool
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
            info_msg: vec![],
            metadata: Default::default(),
            result_type: ClientResultType::Query,
            query_process : ClientResultQueryProcess::line_by_line_compressed,
            debug
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
        self.info_msg.push(
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

    pub fn end_print(self) {
        // Debug output, serialize directly
        if self.debug {
            if self.result_type == ClientResultType::Query {
                let result = serde_json::to_string(&QueryResult::from(self)).unwrap_or("{}".to_string());
                println!("query:{}", &result);
            } else {
                let result = serde_json::to_string(&self).unwrap_or("{}".to_string());
                println!("result:{}", &result);
            }
        } else {
            // Otherwise, output based on conditions
            match self.result_type {
                ClientResultType::Query => {
                    // Process all info messages
                    let infos = self.info_msg;
                    let mut result = String::new();
                    let mut remain : i32 = infos.len() as i32 - 1;
                    for info in infos {
                        result.push_str((self.query_process)(info, remain).as_str());
                        remain -= 1;
                    }
                    println!("{}", &result);
                }
                Fail => {
                    let result = format!("{} ({} errs, {} warns)", "[ Fail ]", self.err_msg.len(), self.warn_msg.len());
                    println!("{}", &result.bright_red());
                }
                ClientResultType::Success => {
                    let warn_count = self.warn_msg.len();
                    if warn_count > 0 {
                        let result = format!("{} ({} warns)", "[ Done ]", self.warn_msg.len());
                        println!("{}", &result.bright_yellow());
                    } else {
                        let result = format!("{}", "[  Ok  ]".green());
                        println!("{}", &result.bright_green());
                    }
                }
            }
        }
    }

    pub fn err_and_end(mut self, msg: &str) {
        self.err(msg);
        self.end_print();
    }

    pub fn has_result(&self) -> bool {
        self.info_msg.len() > 0 || self.warn_msg.len() > 0 || self.err_msg.len() > 0
    }

    pub fn combine(&mut self, other: ClientResult) -> Result<(), ()> {
        // Cannot combine when type is Query
        if self.result_type == ClientResultType::Query || other.result_type == ClientResultType::Query {
            return Err(());
        }

        if other.result_type == Fail {
            self.result_type = Fail;
        }

        for info in other.info_msg {
            self.info_msg.push(info);
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
            query: value.info_msg,
            metadata: value.metadata,
        }
    }
}