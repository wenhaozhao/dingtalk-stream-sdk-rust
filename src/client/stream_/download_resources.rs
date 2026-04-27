use crate::frames::down_message::callback_message::{PayloadFile, PayloadPicture, PayloadVideo};
use crate::DingTalkStream;
use anyhow::anyhow;
use async_trait::async_trait;
use log::info;
use serde_json::json;
use std::ops::Deref;
use std::path::PathBuf;
use url::Url;

#[async_trait]
pub trait DingtalkResource {
    type T;
    async fn fetch(
        &self,
        dingtalk: &DingTalkStream,
        save_to: PathBuf,
    ) -> crate::Result<(PathBuf, Self::T)>;
}

#[cfg(feature = "image")]
type Picture = image::DynamicImage;

#[cfg(not(feature = "image"))]
type Picture = Vec<u8>;

#[async_trait]
impl DingtalkResource for PayloadPicture {
    type T = Picture;

    async fn fetch(
        &self,
        dingtalk: &DingTalkStream,
        save_to_dir: PathBuf,
    ) -> crate::Result<(PathBuf, Self::T)> {
        if !save_to_dir.exists() {
            tokio::fs::create_dir_all(&save_to_dir).await?;
        }
        if save_to_dir.is_file() {
            return Err(anyhow!("save_to_dir is a file"));
        }
        let filepath = save_to_dir.join(format!(
            "{}.png",
            format!("{:x}", md5::compute(&self.download_code))
        ));
        if filepath.exists() {
            let bytes = tokio::fs::read(&filepath).await?;
            #[cfg(feature = "image")]
            return Ok((filepath, image::load_from_memory(&bytes)?));
            #[cfg(not(feature = "image"))]
            return Ok((filepath, bytes));
        }
        let download_url = fetch_download_url(dingtalk, &self.download_code).await?;
        let bytes = dingtalk
            .http_client
            .get(download_url)
            .send()
            .await?
            .bytes()
            .await?;

        #[cfg(feature = "image")]
        let image = {
            use std::io::Cursor;
            let image = image::load_from_memory(&bytes)?;
            let mut cursor = Cursor::new(vec![]);
            image.write_to(&mut cursor, image::ImageFormat::Png)?;
            tokio::fs::write(&filepath, cursor.into_inner()).await?;
            image
        };
        #[cfg(not(feature = "image"))]
        let image = {
            tokio::fs::write(&filepath, bytes.as_ref()).await?;
            bytes.to_vec()
        };
        info!("Downloaded image to {}", filepath.display());
        Ok((filepath, image))
    }
}

#[async_trait]
impl DingtalkResource for PayloadVideo {
    type T = Vec<u8>;

    async fn fetch(
        &self,
        dingtalk: &DingTalkStream,
        save_to_dir: PathBuf,
    ) -> crate::Result<(PathBuf, Self::T)> {
        if !save_to_dir.exists() {
            tokio::fs::create_dir_all(&save_to_dir).await?;
        }
        if save_to_dir.is_file() {
            return Err(anyhow!("save_to_dir is a file"));
        }
        let filepath = save_to_dir.join(format!(
            "{}.{}",
            format!("{:x}", md5::compute(&self.download_code)),
            self.video_type
        ));
        if filepath.exists() {
            let bytes = tokio::fs::read(&filepath).await?;
            return Ok((filepath, bytes));
        }
        let download_url = fetch_download_url(dingtalk, &self.download_code).await?;
        let bytes = dingtalk
            .http_client
            .get(download_url)
            .send()
            .await?
            .bytes()
            .await?;
        tokio::fs::write(&filepath, bytes.as_ref()).await?;
        info!(
            "Downloaded {} video to {}",
            self.download_code,
            filepath.display()
        );
        Ok((filepath, bytes.to_vec()))
    }
}

#[async_trait]
impl DingtalkResource for PayloadFile {
    type T = Vec<u8>;

    async fn fetch(
        &self,
        dingtalk: &DingTalkStream,
        save_to_dir: PathBuf,
    ) -> crate::Result<(PathBuf, Self::T)> {
        if !save_to_dir.exists() {
            tokio::fs::create_dir_all(&save_to_dir).await?;
        }
        if save_to_dir.is_file() {
            return Err(anyhow!("save_to_dir is a file"));
        }
        let filepath = save_to_dir.join(format!(
            "{}_{}",
            format!("{:x}", md5::compute(&self.download_code)),
            self.file_name
        ));
        if filepath.exists() {
            let bytes = tokio::fs::read(&filepath).await?;
            return Ok((filepath, bytes));
        }
        let download_url = fetch_download_url(dingtalk, &self.download_code).await?;
        let bytes = dingtalk
            .http_client
            .get(download_url)
            .send()
            .await?
            .bytes()
            .await?;
        tokio::fs::write(&filepath, bytes.as_ref()).await?;
        info!(
            "Downloaded {} file to {}",
            self.download_code,
            filepath.display()
        );
        Ok((filepath, bytes.to_vec()))
    }
}

async fn fetch_download_url(dingtalk: &DingTalkStream, download_code: &str) -> crate::Result<Url> {
    let access_token = dingtalk.get_access_token().await?;
    let response = dingtalk
        .http_client
        .post(crate::MESSAGE_FILES_DOWNLOAD_URL)
        .header("x-acs-dingtalk-access-token", access_token.deref())
        .header("Content-Type", "application/json")
        .json(&json!({
            "robotCode": &dingtalk.credential.client_id,
            "downloadCode": download_code,
        }))
        .send()
        .await?;
    let code = response.status();
    if response.status().is_success() {
        let json = response.json::<serde_json::Value>().await?;
        let download_url = json.get("downloadUrl").and_then(|it| it.as_str());
        info!(
            "Get download url by download_code: {download_code}, download_url: {}",
            download_url.unwrap_or("None")
        );
        Ok(download_url
            .ok_or(anyhow!("download_url is not found"))?
            .try_into()?)
    } else {
        Err(anyhow!(
            "Failed to download file with unexpected http-code: {}, download_code: {}",
            code,
            download_code
        ))
    }
}
