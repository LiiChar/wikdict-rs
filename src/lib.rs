mod database;
mod downloader;
mod language;
mod pair;

pub use database::{
    DatabaseError, SimpleTranslation, Translation, TranslationGrouped, WikDictDatabase,
};
pub use downloader::{DownloadError, DownloadProgress, WikDictDownloader, WikDictVersion};
pub use language::Language;
pub use pair::{Dictionary, ALL_DICTIONARIES};

pub const DEFAULT_VERSION: &str = "2_2026-06";
pub const DOWNLOAD_BASE_URL: &str = "https://download.wikdict.com/dictionaries/sqlite";

pub fn dictionary_url(version: &str, dictionary: Dictionary) -> String {
    format!("{DOWNLOAD_BASE_URL}/{version}/{}", dictionary.file_name())
}
