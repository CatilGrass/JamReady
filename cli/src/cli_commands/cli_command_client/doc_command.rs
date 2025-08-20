use crate::cli_commands::client::DocArgs;
use crate::data::client_result::{ClientResult, ClientResultQueryProcess};
use crate::help::help_docs::get_help_docs;

pub async fn client_doc (args: DocArgs) {
    let mut query = ClientResult::query(ClientResultQueryProcess::direct).await;
    query.log(get_help_docs(args.doc_name.as_str()).as_str());
    query.end_print();
}