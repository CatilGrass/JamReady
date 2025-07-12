use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};

pub trait LocalArchive: Serialize + for<'a> Deserialize<'a> + Default {
    type DataType: Serialize + for<'a> Deserialize<'a> + Default;

    fn relative_path() -> String;

    /// 加载
    fn read() -> Self::DataType where Self: Sized {
        Self::read_from(Self::relative_path())
    }

    fn read_from(path: String) -> Self::DataType where Self: Sized {

        let file = current_dir().unwrap().join(path);

        // 文件不存在
        if !file.exists() {

            // 创建新的结构
            let data = Self::DataType::default();

            // 创建需要的目录
            create_paths();

            // 序列化并写入磁盘
            let content = serde_yaml::to_string(&data).unwrap();
            if let Ok(mut created) = File::create(&file) {
                created.write_all(content.as_bytes()).unwrap();
            }

            // 返回新的值
            data
        } else {

            // 文件存在
            // 加载文件
            let file = File::open(&file).unwrap();
            let reader = BufReader::new(file);

            // 反序列化并返回
            serde_yaml::from_reader(reader).unwrap_or(Self::DataType::default())
        }
    }

    /// 更新工作区
    fn update(val: &Self::DataType) where Self: Sized {
        Self::update_to(val, Self::relative_path())
    }

    /// 更新工作区
    fn update_to(val: &Self::DataType, path: String) where Self: Sized {

        // 加载文件，并序列化成文本
        let file = current_dir().unwrap().join(path);
        let content = serde_yaml::to_string(&val).unwrap();

        // 创建需要的目录
        create_paths();

        // 将序列化的结果存入磁盘
        let _ = File::create(file).unwrap().write_all(content.as_bytes());
    }
}

/// 创建需要的目录
fn create_paths () {
    let paths = vec![
        env!("PATH_WORKSPACE_ROOT"),
        env!("PATH_PARAMETERS"),
        env!("PATH_DATABASE_CONFIG_ARCHIVE"),
    ];

    for path in paths {
        let dir = current_dir().unwrap().join(path);
        create_dir_all(dir).unwrap_or_default();
    }
}