use crate::data::member::{Member, MemberDuty};
use crate::service::messages::ServerMessage::{Deny, Pass};
use crate::service::service_utils::{read_msg, send_msg};
use tokio::net::TcpStream;

/// Verifies client authentication with the server
pub async fn verify(stream: &mut TcpStream) -> bool {
    match read_msg(stream).await {
        Pass => true,
        Deny(message) => {
            eprintln!("Authentication failed: {}", message);
            false
        }
        _ => false
    }
}

/// Verifies if member has required duty privileges
pub async fn verify_duty(stream: &mut TcpStream, member: &Member, duty: MemberDuty) -> bool {
    if member.member_duties.contains(&duty) {
        send_msg(stream, &Pass).await;
        true
    } else {
        let error_msg = format!("Insufficient privileges: \"{:?}\" duty required", duty);
        send_msg(stream, &Deny(error_msg)).await;
        false
    }
}