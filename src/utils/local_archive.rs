use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::{AsyncReadExt};
use std::env::current_dir;

#[async_trait]
pub trait LocalArchive: Serialize + for<'a> Deserialize<'a> + Default {
    type DataType: Serialize + for<'a> Deserialize<'a> + Default + Send + Sync;

    fn relative_path() -> String;

    async fn read() -> Self::DataType
    where
        Self: Sized + Send + Sync,
    {
        Self::read_from(Self::relative_path()).await
    }

    async fn read_from(path: String) -> Self::DataType
    where
        Self: Sized + Send + Sync,
    {
        let file_path = current_dir().unwrap().join(&path);

        // Check if file exists
        match fs::metadata(&file_path).await {
            Ok(_) => {
                // Open file
                let mut file = fs::File::open(&file_path).await.unwrap();
                let mut contents = String::new();

                // Read contents
                file.read_to_string(&mut contents).await.unwrap();

                // Deserialize
                serde_yaml::from_str(&contents).unwrap_or_default()
            }
            Err(_) => {
                // Return default value when file doesn't exist
                Self::DataType::default()
            }
        }
    }

    async fn update(val: &Self::DataType)
    where
        Self: Sized + Send + Sync,
    {
        Self::update_to(val, Self::relative_path()).await
    }

    async fn update_to(val: &Self::DataType, path: String)
    where
        Self: Sized + Send + Sync,
    {
        // Ensure directory exists
        create_paths().await;

        let file_path = current_dir().unwrap().join(&path);
        let contents = serde_yaml::to_string(val).unwrap();

        // Write to file
        fs::write(&file_path, contents).await.unwrap();
    }
}

async fn create_paths() {
    let paths = vec![
        env!("PATH_WORKSPACE_ROOT"),
        env!("PATH_PARAMETERS"),
        env!("PATH_DATABASE_CONFIG_ARCHIVE"),
        env!("PATH_CACHE"),
    ];

    for path in paths {
        let dir = current_dir().unwrap().join(path);
        fs::create_dir_all(dir).await.unwrap_or_default();
    }
}