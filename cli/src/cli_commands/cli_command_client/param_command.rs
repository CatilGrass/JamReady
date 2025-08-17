use crate::cli_commands::client::ParamArgs;
use crate::data::client_result::{ClientResult, ClientResultQueryProcess};
use crate::data::parameters::{erase_parameter, parameters, read_parameter, write_parameter};

pub async fn client_param(args: ParamArgs) {
    if let Some(key) = args.key {
        match args.value {
            None => client_query_param(key),
            Some(content) => if content.trim() == "null" || content.trim() == "none" {
                erase_parameter(key)
            } else {
                write_parameter(key, content)
            }
        }
    } else {
        let mut result = ClientResult::query(ClientResultQueryProcess::line_by_line).await;
        for parameter in parameters() {
            let parameter = parameter
                .split("/")
                .last().unwrap_or("")
                .to_string();
            if parameter.is_empty() { continue }
            result.log(format!("{} = \"{}\"",
                               parameter.clone(),
                               read_parameter(parameter)
                                   .unwrap_or("".to_string())
                                   .replace("\n", "\\n")
                                   .replace("\t", "\\t")
                                   .replace("\r", "\\r")
            ).as_str());
        }
        result.end_print();
    }
}

/// 查询参数
fn client_query_param(param_name: String) {
    print!("{}", read_parameter(param_name.clone()).unwrap_or("".to_string()));
}