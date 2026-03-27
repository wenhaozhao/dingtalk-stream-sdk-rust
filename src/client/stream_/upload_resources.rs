use crate::DingTalkStream;
use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::multipart::Part;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use url::Url;

#[async_trait]
pub trait DingTalkMedia {
    async fn upload(&self, dingtalk: &DingTalkStream) -> crate::Result<MediaUploadResult>;
}

#[async_trait]
impl<M> DingTalkMedia for M
where
    M: TryInto<DingTalkMedia_> + Clone + Send + Sync,
{
    async fn upload(&self, dingtalk: &DingTalkStream) -> crate::Result<MediaUploadResult> {
        let media = self
            .clone()
            .try_into()
            .map_err(|_| anyhow!("Failed to convert to DingTalkMedia_"))?;
        Ok(media.upload_(dingtalk).await?)
    }
}
#[derive(Debug, Clone)]
pub enum DingTalkMedia_ {
    Image(MediaImage),
    Voice(MediaVoice),
    File(MediaFile),
    Video(MediaVideo),
}

impl Deref for DingTalkMedia_ {
    type Target = MediaContent;

    fn deref(&self) -> &Self::Target {
        match self {
            DingTalkMedia_::Image(content) => content,
            DingTalkMedia_::Voice(content) => content,
            DingTalkMedia_::File(content) => content,
            DingTalkMedia_::Video(content) => content,
        }
    }
}

impl DingTalkMedia_ {
    pub fn type_(&self) -> MediaType {
        match self {
            DingTalkMedia_::Image(_) => MediaType::Image,
            DingTalkMedia_::Voice(_) => MediaType::Voice,
            DingTalkMedia_::File(_) => MediaType::File,
            DingTalkMedia_::Video(_) => MediaType::Video,
        }
    }

    async fn as_bytes(&self) -> crate::Result<Vec<u8>> {
        match self {
            DingTalkMedia_::Image(content) => content.as_bytes().await,
            DingTalkMedia_::Voice(content) => content.as_bytes().await,
            DingTalkMedia_::File(content) => content.as_bytes().await,
            DingTalkMedia_::Video(content) => content.as_bytes().await,
        }
    }
}

impl DingTalkMedia_ {
    async fn upload_(&self, dingtalk: &DingTalkStream) -> crate::Result<MediaUploadResult> {
        let access_token = dingtalk.get_access_token().await?;
        let bytes = self.as_bytes().await?;

        let filename = self.filename()?;
        let form = reqwest::multipart::Form::new()
            .text("type", self.type_().to_string())
            .part(
                "media",
                Part::bytes(bytes).file_name(filename).headers({
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        "Content-Type",
                        HeaderValue::from_static("application/octet-stream"),
                    );
                    headers
                }),
            );
        let result = dingtalk
            .http_client
            .post(crate::MEDIA_UPLOAD_URL)
            .query(&[("access_token", access_token.deref())])
            .multipart(form)
            .send()
            .await?
            .json::<MediaUploadResult>()
            .await?;
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct MediaImage(MediaContent);

impl<C: Into<MediaContent>> From<C> for MediaImage {
    fn from(content: C) -> Self {
        MediaImage(content.into())
    }
}

impl Deref for MediaImage {
    type Target = MediaContent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<MediaImage> for DingTalkMedia_ {
    fn from(value: MediaImage) -> Self {
        Self::Image(value)
    }
}

#[derive(Debug, Clone)]
pub struct MediaVoice(MediaContent);
impl<C: Into<MediaContent>> From<C> for MediaVoice {
    fn from(content: C) -> Self {
        MediaVoice(content.into())
    }
}

impl Deref for MediaVoice {
    type Target = MediaContent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<MediaVoice> for DingTalkMedia_ {
    fn from(value: MediaVoice) -> Self {
        Self::Voice(value)
    }
}

#[derive(Debug, Clone)]
pub struct MediaFile(MediaContent);

impl<C: Into<MediaContent>> From<C> for MediaFile {
    fn from(content: C) -> Self {
        MediaFile(content.into())
    }
}

impl Deref for MediaFile {
    type Target = MediaContent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<MediaFile> for DingTalkMedia_ {
    fn from(value: MediaFile) -> Self {
        Self::File(value)
    }
}

#[derive(Debug, Clone)]
pub struct MediaVideo(MediaContent);
impl<C: Into<MediaContent>> From<C> for MediaVideo {
    fn from(content: C) -> Self {
        MediaVideo(content.into())
    }
}

impl Deref for MediaVideo {
    type Target = MediaContent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<MediaVideo> for DingTalkMedia_ {
    fn from(value: MediaVideo) -> Self {
        Self::Video(value)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MediaType {
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "voice")]
    Voice,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "video")]
    Video,
}

impl FromStr for MediaType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "image" | "img" => Ok(MediaType::Image),
            "voice" => Ok(MediaType::Voice),
            "file" => Ok(MediaType::File),
            "video" => Ok(MediaType::Video),
            _ => Err(anyhow::anyhow!("Invalid media type: {}", s)),
        }
    }
}

impl Deref for MediaType {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            MediaType::Image => "image",
            MediaType::Voice => "voice",
            MediaType::File => "file",
            MediaType::Video => "video",
        }
    }
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.deref())
    }
}

#[derive(Debug, Clone)]
pub enum MediaContent {
    Bytes { filename: String, bytes: Vec<u8> },
    Filepath(PathBuf),
    Url { filename: String, url: Url },
}

impl MediaContent {
    fn filename(&self) -> crate::Result<String> {
        match self {
            MediaContent::Bytes { filename, .. } => Ok(filename.to_string()),
            MediaContent::Filepath(filepath) => filepath
                .file_name()
                .map(|it| it.to_string_lossy().to_string())
                .ok_or(anyhow!("parse filename failed")),
            MediaContent::Url { filename, .. } => Ok(filename.to_string()),
        }
    }
}

impl MediaContent {
    async fn as_bytes(&self) -> crate::Result<Vec<u8>> {
        match self {
            MediaContent::Bytes { bytes, .. } => Ok(bytes.clone()),
            MediaContent::Filepath(path) => Ok(tokio::fs::read(path).await?),
            MediaContent::Url { url, .. } => {
                let response = reqwest::get(url.as_str()).await?;
                Ok(response.bytes().await.map(|bytes| bytes.to_vec())?)
            }
        }
    }
}

impl<Filename: AsRef<str>> From<(Filename, Vec<u8>)> for MediaContent {
    fn from((filename, bytes): (Filename, Vec<u8>)) -> Self {
        MediaContent::Bytes {
            filename: filename.as_ref().to_string(),
            bytes,
        }
    }
}

impl From<PathBuf> for MediaContent {
    fn from(value: PathBuf) -> Self {
        MediaContent::Filepath(value.into())
    }
}

impl From<&Path> for MediaContent {
    fn from(value: &Path) -> Self {
        value.to_path_buf().into()
    }
}

impl<Filename: AsRef<str>> From<(Filename, Url)> for MediaContent {
    fn from((filename, value): (Filename, Url)) -> Self {
        MediaContent::Url {
            filename: filename.as_ref().to_string(),
            url: value,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaUploadResult {
    pub errcode: i32,
    pub errmsg: String,
    #[serde(flatten)]
    pub media: Option<Media>,
}

impl Default for MediaUploadResult {
    fn default() -> Self {
        Self {
            errcode: -1,
            errmsg: "unknown".to_string(),
            media: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub media_id: String,
    #[serde(rename = "type")]
    pub r#type: MediaType,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaId(String);

impl<T, C> TryFrom<(T, C)> for DingTalkMedia_
where
    T: TryInto<MediaType>,
    C: Into<MediaContent>,
{
    type Error = anyhow::Error;

    fn try_from((type_, content): (T, C)) -> Result<Self, Self::Error> {
        let type_ = type_
            .try_into()
            .map_err(|_| anyhow!("unexpected media-type"))?;
        let content = content.into();
        match type_ {
            MediaType::Image => Ok(Self::Image(content.into())),
            MediaType::Voice => Ok(Self::Voice(content.into())),
            MediaType::File => Ok(Self::File(content.into())),
            MediaType::Video => Ok(Self::Video(content.into())),
        }
    }
}
