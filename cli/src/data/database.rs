use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::current_dir;
use std::path::PathBuf;
use uuid::Uuid;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::process_path_text;
use crate::data::database::VirtualFileState::{Available, Lock};
use crate::data::member::Member;
use crate::data::workspace::Workspace;

/// # Database - 文件数据库
/// 数据库是 Jam Ready 中的文件存储站
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct Database {

    /// 所有文件 (Uid, 文件)
    virtual_files: HashMap<String, VirtualFile>,

    /// 文件目录和 Uid 的映射 (目录, Uuid)
    virtual_uuids: HashMap<String, String>
}

/// 虚拟文件
/// 它用于映射工作区内的一个确定位置，并记录其版本、描述、动态等信息
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct VirtualFile {

    /// 目录
    path: String,

    /// 映射到的本地文件
    real: String,

    /// 该文件的版本号
    version: u32,

    /// 该文件的修改历史
    change_histories: HashMap<u32, String>,

    /// 该文件的历史版本映射
    real_histories: HashMap<u32, String>,

    /// 文件状态
    state: VirtualFileState,

    /// 锁是否长期持有
    longer_lock: bool
}

/// 虚拟文件状态
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub enum VirtualFileState {

    /// 空闲的 (表示可读写)
    Available,

    /// 正在被编辑 (参数为成员的 Uid，表示持有者可读写，其他人只读)
    Lock(String)
}

/// 构造数据库
impl Default for Database {

    fn default() -> Self {
        Self {
            virtual_files: HashMap::new(),
            virtual_uuids: HashMap::new()
        }
    }
}

/// 加载和更新功能
impl LocalArchive for Database {
    type DataType = Database;

    fn relative_path() -> String {
        env!("FILE_DATABASE").to_string()
    }
}

impl Database {

    /// 目录是否存在 (是否有目录映射)
    pub fn contains_path(&self, path: &str) -> bool {
        self.virtual_uuids.contains_key(&process_path_text(path.to_string()))
    }

    /// 移除某个目录的映射使其无法通过目录找到
    pub fn remove_file_map(&mut self, path: String) -> Result<String, ()> {
        let path = process_path_text(path);
        // 拿到需要移除的 Uuid
        let uuid = self.virtual_uuids.get(path.as_str());
        // 需要移除的目录已经映射了 Uuid
        if let Some(uuid) = uuid {
            // 通过 Uuid 确实能找到文件
            let file = self.virtual_files.get_mut(uuid);
            if let Some(file) = file {
                // 强制丢掉文件的锁
                file.throw_locker();

                // 移除文件的目录映射
                file.path = "".to_string();
                if let Some(uuid) = self.virtual_uuids.remove(path.as_str()) {
                    return Ok(uuid)
                }
            }
        }
        Err(())
    }

    /// 为某个 Uuid 重建目录映射
    pub fn rebuild_path_to_uuid(&mut self, uuid: String, path: String) -> Result<(), ()> {
        let path = process_path_text(path);
        // 需要重建的目录不存在
        if ! self.contains_path(path.as_str()) {
            // 该 Uuid 存在
            if let Some(file) = self.virtual_files.get_mut(&uuid) {
                // 该文件未绑定目录
                if file.path.is_empty() {
                    // 为文件指定目录
                    file.path = path.clone();
                    // 建立目录映射
                    self.virtual_uuids.insert(uuid, path);
                    return Ok(())
                }
            }
        }
        Err(())
    }

    /// 获得所有文件的引用
    pub fn files(&self) -> Vec<&VirtualFile> {
        let mut file_list = Vec::new();
        for (_uuid, file) in self.virtual_files.iter() {

            // 存在目录才会被列出
            if !file.path.trim().is_empty() {
                file_list.push(file);
            }
        }
        file_list
    }

    /// 获得所有文件的引用 (可变)
    pub fn files_mut(&mut self) -> Vec<&mut VirtualFile> {
        let mut file_list = Vec::new();
        for (_uuid, file) in self.virtual_files.iter_mut() {
            file_list.push(file);
        }
        file_list
    }

    /// 插入虚拟文件
    /// 失败则将构建的数据还回去
    pub fn insert_virtual_file(&mut self, file: VirtualFile) -> Result<bool, VirtualFile> {

        // 判断是否有相同路径的虚拟文件
        if self.contains_path(file.path.as_str().trim()) {
            // 有，则插入失败
            return Err(file);
        }

        // 无，则建立 Uid，修改数据库
        let uuid = Uuid::new_v4();
        self.virtual_files.insert(uuid.to_string(), file.clone());
        self.virtual_uuids.insert(file.path, uuid.to_string());

        Ok(true)
    }

    /// 通过 Uuid 获得虚拟文件
    pub fn file_with_uuid(&self, uuid: String) -> Option<&VirtualFile> {
        self.virtual_files.get(&uuid)
    }

    /// 通过目录获得虚拟文件
    pub fn file(&self, path: String) -> Option<&VirtualFile> {
        let uuid = self.virtual_uuids.get(path.as_str());
        if let Some(uuid) = uuid {
            self.file_with_uuid(uuid.clone())
        } else {
            None
        }
    }

    /// 搜索文件
    pub fn search_file(&self, search: String) -> Option<&VirtualFile> {
        if let Some(file) = self.file_with_uuid(search.trim().to_string()) {
            return Some(file);
        } else if let Some(file) = self.file(process_path_text(search)) {
            return Some(file)
        }
        None
    }

    /// 搜索文件 (可变)
    pub fn search_file_mut(&mut self, search: String) -> Option<&mut VirtualFile> {
        if let Some(_) = self.file_with_uuid(search.trim().to_string()) {
            return self.file_mut_with_uuid(search);
        } else if let Some(_) = self.file(process_path_text(search.clone())) {
            return self.file_mut(search);
        }
        None
    }

    /// 通过 Uuid 获得虚拟文件 (可变)
    pub fn file_mut_with_uuid(&mut self, uuid: String) -> Option<&mut VirtualFile> {
        self.virtual_files.get_mut(&uuid)
    }

    /// 通过目录获得虚拟文件 (可变)
    pub fn file_mut(&mut self, path: String) -> Option<&mut VirtualFile> {
        let uuid = self.virtual_uuids.get(path.as_str());
        if let Some(uuid) = uuid {
            self.file_mut_with_uuid(uuid.clone())
        } else {
            None
        }
    }

    /// 修改文件路径
    pub fn move_file(&mut self, old_path: String, new_path: String) -> Result<(), ()> {
        let uuid = self.virtual_uuids.get(&process_path_text(old_path));
        if let Some(uuid) = uuid {
            self.move_file_with_uuid(uuid.clone(), new_path)
        } else {
            Err(())
        }
    }

    /// 通过 Uuid 修改路径
    pub fn move_file_with_uuid(&mut self, uuid: String, new_path: String) -> Result<(), ()> {
        // 处理新目录字符串
        let new_path = process_path_text(new_path);

        // 新目录不存在时执行
        if ! self.contains_path(new_path.as_str()) {

            // 获得文件
            let file = self.virtual_files.get_mut(&uuid);
            if let Some(file) = file {

                // 移除目录映射
                self.virtual_uuids.remove(file.path.as_str());

                // 修改自身目录
                file.path = new_path.clone();

                // 重建目录映射
                self.virtual_uuids.insert(new_path, uuid);

                // 若不是长期锁，则直接丢弃
                if !file.is_longer_lock_unchecked() {
                    file.throw_locker();
                }

                return Ok(())
            }
            Err(())
        } else {
            Err(())
        }
    }

    /// 清理历史版本
    pub fn clean_histories(&mut self) {
        for (_uuid, file) in self.virtual_files.iter_mut() {
            file.change_histories = HashMap::new();
            file.real_histories = HashMap::new();
        }
    }

    /// 获得某个目录的 Uuid
    pub fn uuid_of_path(&self, path: String) -> Option<String> {
        if let Some(uuid) = self.virtual_uuids.get(path.as_str()) {
            return Some(uuid.to_string());
        }
        None
    }
}

impl VirtualFile {

    /// 通过目录新建
    pub fn new(path: String) -> Self {
        let mut new = Self {
            path: process_path_text(path),
            real: "".to_string(),
            version: 0,
            change_histories: Default::default(),
            real_histories: Default::default(),
            state: Available,
            longer_lock: false
        };

        // 添加第零版本数据
        new.change_histories.insert(0, "First Version".to_string());
        new.real_histories.insert(0, "".to_string());

        new
    }

    /// 获得文件的名称
    pub fn name(&self) -> Option<String> {
        if let Some(name) = self.path.split('/').last() {
            return Some(name.to_string())
        }
        None
    }

    /// 获得文件的目录
    pub fn path(&self) -> String {
        self.path.clone()
    }

    /// 获得文件的真实目录
    pub fn real_path(&self) -> String {
        self.real.clone()
    }

    /// 获得文件的真实目录
    pub fn version(&self) -> u32 {
        self.version
    }

    /// 尝试获得对应的客户端文件
    pub fn client_path(&self) -> Option<PathBuf> {
        match current_dir() {
            Ok(current) => {
                let current = current.join(self.path.as_str());
                Some(current)
            }
            Err(_) => None
        }
    }

    /// 获得对应的服务端文件
    pub fn server_path(&self) -> Option<PathBuf> {
        match current_dir() {
            Ok(current) => {
                let current = current.join(env!("PATH_DATABASE")).join(self.real.as_str());
                Some(current)
            }
            Err(_) => None
        }
    }

    /// 获得对应的服务端文件
    pub fn server_path_version(&self, version: u32) -> Option<PathBuf> {
        match current_dir() {
            Ok(current) => {
                if let Some(real) = self.real_histories.get(&version) {
                    let current = current.join(env!("PATH_DATABASE")).join(real);
                    return Some(current);
                }
                None
            }
            Err(_) => None
        }
    }

    /// 获得对应的服务端文件
    pub fn server_path_temp(&self, temp_real: String) -> Option<PathBuf> {
        match current_dir() {
            Ok(current) => {
                let current = current.join(env!("PATH_DATABASE")).join(temp_real);
                Some(current)
            }
            Err(_) => None
        }
    }

    /// 更新实际地址
    pub fn update(&mut self, new_real_path: String, changes_info: String) {

        // 版本号前进一位
        self.version += 1;

        // 添加现版本数据
        self.change_histories.insert(self.version, changes_info);
        self.real_histories.insert(self.version, new_real_path.clone());

        // 更新当前实际目录
        self.real = new_real_path;
    }

    /// 回滚至指定版本
    pub fn rollback_to_version(&mut self, version: u32) -> bool {
        if let Some(old_real) = self.real_histories.get(&version) {
            self.version = version;
            self.real = old_real.clone();

            // 若不是长期锁，则直接丢弃
            if !self.is_longer_lock_unchecked() {
                self.throw_locker();
            }

            return true;
        }
        false
    }

    /// 尝试给成员获得锁 (通过 Uuid)
    pub async fn give_uuid_locker(&mut self, member_uuid: String, longer: bool) -> bool {
        let workspace = Workspace::read().await;
        if let Some(server) = workspace.server {

            // 自身为空闲
            if self.state == Available {

                // 存在该成员
                if server.members.contains_key(&member_uuid) {

                    // 锁定
                    self.state = Lock(member_uuid);
                    self.longer_lock = longer;
                    return true;
                }
            } else if let Lock(guid) = &self.state {

                // 或者该成员已持有锁
                if guid == &member_uuid.trim() {
                    return true;
                }
            }
        }
        false
    }

    /// 尝试给成员获得锁 (通过 Uuid)
    pub async fn give_locker(&mut self, member: &Member, longer: bool) -> bool {
        let workspace = Workspace::read().await;
        if let Some(server) = workspace.server {

            // 自身为空闲
            if self.state == Available {

                // 获得 Uuid
                let uuid = server.member_uuids.get(&member.member_name);
                if let Some(uuid) = uuid {
                    // 锁定
                    self.state = Lock(uuid.clone());
                    self.longer_lock = longer;
                    return true;
                }

            } else if let Lock(locker_owner) = &self.state {

                // 获得 Uuid
                let uuid = server.member_uuids.get(&member.member_name);
                if let Some(member_uuid) = uuid {

                    // 或者该成员已持有锁
                    if locker_owner == member_uuid {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 丢掉自身的锁
    pub fn throw_locker(&mut self) {
        self.state = Available;
        self.longer_lock = false;
    }

    /// 获得锁的主人
    pub async fn get_locker_owner(&self) -> Option<(String, Member)> {
        if let Lock(uuid) = &self.state {
            let workspace = Workspace::read().await;
            if let Some(mut server) = workspace.server {
                if let Some(member) = server.members.remove(uuid.trim()) {
                    return Some((uuid.clone(), member));
                }
            }
        }
        None
    }

    /// 获得锁的主人 (适用于客户端环境)
    pub fn get_locker_owner_uuid(&self) -> Option<String> {
        if let Lock(uuid) = &self.state {
            return Some(uuid.clone());
        }
        None
    }

    /// 判断锁是否为长期锁
    pub fn is_longer_lock(&self) -> Option<bool> {
        if let Lock(_) = &self.state {
            return Some(self.longer_lock);
        }
        None
    }

    /// 判断锁是否为长期锁
    pub fn is_longer_lock_unchecked(&self) -> bool {
        self.longer_lock
    }
}