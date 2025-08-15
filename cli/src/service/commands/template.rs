use std::sync::Arc;
use async_trait::async_trait;
use tokio::net::TcpStream;
use crate::data::database::Database;
use crate::data::member::Member;
use crate::service::jam_command::Command;

pub struct TemplateCommand;

#[async_trait]
impl Command for TemplateCommand {

    async fn local(&self, _stream: &mut TcpStream, _args: Vec<&str>) -> Option<ClientResult> {
        let mut command_result = ClientResult::result().await;
        return Some(command_result);
    }

    async fn remote(&self,
                    _stream: &mut TcpStream, _args: Vec<&str>,
                    _member: (String, &Member), _database: Arc<Mutex<Database>>) {
        false
    }
}