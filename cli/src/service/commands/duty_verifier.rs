use crate::data::member::{Member, MemberDuty};
use crate::service::messages::ServerMessage;
use crate::service::messages::ServerMessage::{Deny, Pass};
use crate::service::service_utils::{read_msg, send_msg};
use tokio::net::TcpStream;

pub async fn verify(stream: &mut TcpStream) -> bool {

    // 接收服务端发来的身份验证信息
    let message: ServerMessage = read_msg(stream).await;
    if let Deny(message) = message {
        eprintln!("{}", message);
        return false
    } else if message == Pass {
        return true
    }
    false
}

pub async fn verify_duty(stream: &mut TcpStream, member: &Member, duty: MemberDuty) -> bool {

    // 检查成员身份，只有对应身份可以继续
    if ! member.member_duties.contains(&duty) {
        send_msg(stream, &Deny(format!("Only \"{:?}\" can execute this command.", duty))).await;
        false
    } else {
        send_msg(stream, &Pass).await;
        true
    }
}