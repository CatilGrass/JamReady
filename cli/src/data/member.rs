use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

/// Member - 成员
/// 成员表示 JamReady 中每个角色的身份
/// 记录了该成员的基本信息
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct Member {

    /// 成员名称
    pub member_name: String,

    /// 成员职责
    pub member_duties: Vec<MemberDuty>,
}

/// 成员职责 (可复合)
#[derive(Serialize, Deserialize, Encode, Decode, EnumIter, Clone, Debug, PartialEq)]
pub enum MemberDuty {

    /// 调试人员（有些操作只有调试人员能做）
    Debugger,

    /// 队长
    Leader,

    /// 开发者
    Developer,

    /// 设计师
    Creator,

    /// 协调者 (策划)
    Producer
}

impl Member {

    /// 新建角色
    pub fn new(member_name: String) -> Self {
        Self {
            member_name,
            member_duties: Vec::new(),
        }
    }

    /// 增加职责
    pub fn add_duty(&mut self, duty: MemberDuty) {

        if !self.member_duties.contains(&duty) {
            self.member_duties.push(duty);
        }
    }

    /// 移除职责
    pub fn remove_duty(&mut self, duty: MemberDuty) {

        let mut i = 0;
        for iter in self.member_duties.iter() {
            if iter == &duty {
                self.member_duties.remove(i);
                return;
            }
            i += 1;
        }
    }
}