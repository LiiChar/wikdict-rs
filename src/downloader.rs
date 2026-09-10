use futures_util::StreamExt;
use reqwest::{Client, StatusCode};
use std::path::{Path, PathBuf};
use tokio::{fs::File, io::AsyncWriteExt};
use crate::{dictionary_url, Dictionary, DEFAULT_VERSION};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WikDictVersion(&'static str);
impl WikDictVersion {
    pub const LATEST: Self = Self(DEFAULT_VERSION);
    pub const fn new(value: &'static str) -> Self { Self(value) }
    pub const fn as_str(self) -> &'static str { self.0 }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DownloadProgress { pub downloaded: u64, pub total: Option<u64> }
impl DownloadProgress {
    pub fn fraction(self) -> Option<f64> { self.total.map(|t| if t == 0 { 1.0 } else { self.downloaded as f64 / t as f64 }) }
}

#[derive(Clone)]
pub struct WikDictDownloader { client: Client, version: WikDictVersion }
impl WikDictDownloader {
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self { client: Client::builder().user_agent(concat!("wikdict-rs/", env!("CARGO_PKG_VERSION"))).build()?, version: WikDictVersion::LATEST })
    }
    pub fn with_version(version: WikDictVersion) -> Result<Self, reqwest::Error> {
        let mut x = Self::new()?; x.version = version; Ok(x)
    }
    pub const fn version(&self) -> WikDictVersion { self.version }
    pub fn url(&self, dictionary: Dictionary) -> String { dictionary_url(self.version.as_str(), dictionary) }

    pub async fn download(&self, dictionary: Dictionary, destination: impl AsRef<Path>) -> Result<PathBuf, DownloadError> {
        self.download_with_progress(dictionary, destination, |_| {}).await
    }

    pub async fn download_with_progress<F>(&self, dictionary: Dictionary, destination: impl AsRef<Path>, mut on_progress: F) -> Result<PathBuf, DownloadError>
    where F: FnMut(DownloadProgress) {
        let destination = destination.as_ref().to_path_buf();
        if let Some(parent) = destination.parent() { tokio::fs::create_dir_all(parent).await?; }
        let partial = destination.with_extension(format!("{}part", destination.extension().and_then(|x| x.to_str()).map(|x| format!("{x}.")).unwrap_or_default()));
        let response = self.client.get(self.url(dictionary)).send().await?;
        if response.status() == StatusCode::NOT_FOUND { return Err(DownloadError::NotFound(dictionary)); }
        let response = response.error_for_status()?;
        let total = response.content_length();
        let mut file = File::create(&partial).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?; file.write_all(&chunk).await?; downloaded += chunk.len() as u64;
            on_progress(DownloadProgress { downloaded, total });
        }
        file.flush().await?; drop(file);
        if destination.exists() { tokio::fs::remove_file(&destination).await?; }
        tokio::fs::rename(&partial, &destination).await?;
        Ok(destination)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("dictionary is not available: {0}")] NotFound(Dictionary),
    #[error("HTTP error: {0}")] Http(#[from] reqwest::Error),
    #[error("I/O error: {0}")] Io(#[from] std::io::Error),
}
