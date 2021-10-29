use std::collections::HashMap;
use std::path::PathBuf;
use rocket::fs::TempFile;
use serde::{Serialize, Deserialize};
use tokio::fs::{self, OpenOptions, create_dir_all};
use tokio::io;

#[derive(Serialize, Deserialize)]
pub struct UpdatePkg {
    pub pkg_url: Option<String>,
    pub version: u32,
    pub mandatory: bool,
    pub update_log: String,
}

type Updates = HashMap<String, Vec<UpdatePkg>>;

pub struct UpdateProject<'r> {
    project_name: &'r str,
    updates: Updates,
}

impl<'r> UpdateProject<'r> {
    #[inline]
    fn update_path(&self) -> PathBuf {
        format!("./update_file/{}/", self.project_name).into()
    }

    #[inline]
    fn version_path(project_name: &str) -> PathBuf {
        format!("./update_file/{}/version.json", project_name).into()
    }

    #[inline]
    pub fn get_download_path(&self, pkg_version: &str, hot_version: u32) -> String {
        format!("update_file/{}/{}/{}.zip", self.project_name, pkg_version, hot_version)
    }

    pub async fn new(project_name: &'r str) -> tokio::io::Result<UpdateProject<'_>> {
        let version_path = Self::version_path(project_name);
        let parent = version_path.parent().unwrap();
        log::info!("{:?}", parent);
        if !parent.exists() {
            create_dir_all(parent).await?;
        }
        let version_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(version_path)
            .await?;
        let updates = serde_json::from_reader(version_file.into_std().await).unwrap_or(HashMap::new());
        Ok(Self {
            project_name,
            updates,
        })
    }

    pub fn get_new(&self, pkg_version: &str, hot_version: u32) -> Option<&UpdatePkg> {
        let versions = self.updates.get(pkg_version)?;
        versions.iter().find(|up| up.version > hot_version)
    }

    pub async fn add(
        &mut self,
        pkg_version: &str,
        temp_file: Option<TempFile<'_>>,
        pkg_url: Option<String>,
        mandatory: bool,
        update_log: &str,
    ) -> io::Result<()> {
        let versions = self.updates.entry(pkg_version.to_string()).or_insert(Vec::new());
        let version = versions.last().map(|up| up.version).unwrap_or(0) + 1;
        let update_pkg = UpdatePkg {
            version,
            pkg_url,
            mandatory,
            update_log: update_log.to_string(),
        };
        versions.push(update_pkg);
        if let Some(mut temp_file) = temp_file {
            let mut upload_path = self.update_path();
            upload_path.push(pkg_version);
            if !upload_path.exists() {
                create_dir_all(&upload_path).await?;
            }
            upload_path.push(format!("{}.zip", version));
            temp_file.persist_to(&upload_path).await?;
        }
        Ok(())
    }

    pub async fn save(&self) -> tokio::io::Result<()> {
        let version_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(Self::version_path(self.project_name)).await?;
        serde_json::to_writer(version_file.into_std().await, &self.updates)?;
        Ok(())
    }

    pub async fn remove(&mut self, pkg_version: &str, hot_version: u32) -> tokio::io::Result<()> {
        if let Some(versions) = self.updates.get_mut(pkg_version) {
            let index = versions.iter().enumerate().find(|(_i, up)| up.version == hot_version);
            if let Some((index, _)) = index {
                versions.remove(index);
                let mut path = self.update_path();
                path.push(format!("{}/{}.zip", pkg_version, hot_version));
                if path.exists() {
                    fs::remove_file(path).await?
                }
            }
        }
        Ok(())
    }
}
