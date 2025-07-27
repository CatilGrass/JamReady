use async_trait::async_trait;
use tokio::net::TcpStream;
use crate::data::database::Database;
use crate::data::member::Member;
use crate::service::jam_command::Command;

pub struct TemplateCommand;

#[async_trait]
impl Command for TemplateCommand {

    async fn local(&self, _stream: &mut TcpStream, _args: Vec<&str>) {

    }

    async fn remote(
        &self, _stream: &mut TcpStream, _args: Vec<&str>,
        (_uuid, _member): (String, &Member), _database: &mut Database)
        -> bool {

        false
    }
}