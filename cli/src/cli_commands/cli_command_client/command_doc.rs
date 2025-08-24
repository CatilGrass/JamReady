use crate::cli_commands::client::DocArgs;
use crate::data::client_result::{ClientResult, ClientResultQueryProcess};
use crate::help::help_docs::get_help_docs;

pub async fn client_doc (args: DocArgs) -> Option<ClientResult> {

    // Create query result
    let mut query = ClientResult::query(ClientResultQueryProcess::direct).await;

    // Insert results
    query.log(get_help_docs(args.doc_name.as_str()).as_str());
    
    Some(query)
}