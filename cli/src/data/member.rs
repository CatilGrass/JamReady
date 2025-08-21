use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

/// Member - Team Member
/// Represents each team member's identity in JamReady
/// Records basic information about the member
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct Member {
    /// Member name
    #[serde(rename = "name")]
    pub member_name: String,

    /// Member responsibilities (can be combined)
    #[serde(rename = "duty")]
    pub member_duties: Vec<MemberDuty>,
}

/// Member responsibilities (can be combined)
#[derive(Serialize, Deserialize, Encode, Decode, EnumIter, Clone, Debug, PartialEq)]
pub enum MemberDuty {
    /// Debugger (some operations are only available to debuggers)
    Debugger,

    /// Team leader
    Leader,

    /// Developer
    Developer,

    /// Designer
    Creator,

    /// Coordinator (Producer)
    Producer
}

impl Member {
    /// Create new member
    pub fn new(member_name: String) -> Self {
        Self {
            member_name,
            member_duties: Vec::new(),
        }
    }

    /// Add responsibility
    pub fn add_duty(&mut self, duty: MemberDuty) {
        if !self.member_duties.contains(&duty) {
            self.member_duties.push(duty);
        }
    }

    /// Remove responsibility
    pub fn remove_duty(&mut self, duty: MemberDuty) {
        if let Some(index) = self.member_duties.iter().position(|d| d == &duty) {
            self.member_duties.remove(index);
        }
    }
}