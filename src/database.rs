use rusqlite::{types::ValueRef, Connection};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("invalid UTF-8")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// Raw aggregated WikDict translation list.
///
/// `trans_list` in the generic SQLite database is not necessarily a list of
/// human-readable translations. It is an aggregated representation of target
/// vocables and can contain values such as:
///
/// `foo | bar | baz`
///
/// or internal WikDict/DBnary vocable identifiers.
#[derive(Debug, Clone, Default)]
pub struct TranslationList {
    pub values: Vec<String>,
}

impl TranslationList {
    pub fn parse(value: impl Into<String>) -> Self {
        let value = value.into();

        Self {
            values: value
                .split(" | ")
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.values.iter().map(String::as_str)
    }
}

#[derive(Debug, Clone)]
pub struct SimpleTranslation {
    pub written_rep: String,

    /// Raw WikDict aggregated translation/vocable list.
    pub trans_list: TranslationList,

    pub max_score: f32,
    pub rel_importance: f32,
}

#[derive(Debug, Clone)]
pub struct TranslationGrouped {
    pub lexentry: Option<String>,
    pub written_rep: String,
    pub min_sense_num: Option<i32>,

    /// Aggregated senses.
    pub sense_list: TranslationList,

    /// Raw aggregated target vocables/translations.
    pub trans_list: TranslationList,

    pub score: f32,
    pub importance: f32,
}

#[derive(Debug, Clone)]
pub struct Translation {
    pub lexentry: Option<String>,
    pub sense_num: Option<i32>,
    pub sense: Option<String>,
    pub written_rep: String,

    /// Raw target vocable/translation.
    pub trans_list: TranslationList,

    pub score: f32,
    pub is_good: bool,
    pub importance: f32,
}

pub struct WikDictDatabase {
    conn: Connection,
}

impl WikDictDatabase {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DatabaseError> {
        Ok(Self {
            conn: Connection::open(path)?,
        })
    }

    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self, DatabaseError> {
        Ok(Self {
            conn: Connection::open_with_flags(
                path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )?,
        })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Looks up an exact word in `simple_translation`.
    ///
    /// This is the preferred lookup for a normal dictionary/reader tooltip.
    pub fn lookup_simple(
        &self,
        word: &str,
    ) -> Result<Vec<SimpleTranslation>, DatabaseError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                written_rep,
                trans_list,
                max_score,
                rel_importance
            FROM simple_translation
            WHERE written_rep = ?1
            ORDER BY max_score DESC
            "#,
        )?;

        let rows = statement.query_map([word], |row| {
            Ok(SimpleTranslation {
                written_rep: row.get(0)?,
                trans_list: TranslationList::parse(
                    value_to_string(row.get_ref(1)?)?,
                ),
                max_score: value_to_f32(row.get_ref(2)?)?,
                rel_importance: value_to_f32(row.get_ref(3)?)?,
            })
        })?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Prefix search in `simple_translation`.
    pub fn search_simple(
        &self,
        prefix: &str,
        limit: u32,
    ) -> Result<Vec<SimpleTranslation>, DatabaseError> {
        let pattern = format!("{prefix}%");

        let mut statement = self.conn.prepare(
            r#"
            SELECT
                written_rep,
                trans_list,
                max_score,
                rel_importance
            FROM simple_translation
            WHERE written_rep LIKE ?1
            ORDER BY max_score DESC
            LIMIT ?2
            "#,
        )?;

        let rows = statement.query_map(
            rusqlite::params![pattern, limit],
            |row| {
                Ok(SimpleTranslation {
                    written_rep: row.get(0)?,
                    trans_list: TranslationList::parse(
                        value_to_string(row.get_ref(1)?)?,
                    ),
                    max_score: value_to_f32(row.get_ref(2)?)?,
                    rel_importance: value_to_f32(row.get_ref(3)?)?,
                })
            },
        )?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Looks up grouped lexical entries.
    pub fn lookup_grouped(
        &self,
        word: &str,
    ) -> Result<Vec<TranslationGrouped>, DatabaseError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                lexentry,
                written_rep,
                min_sense_num,
                sense_list,
                trans_list,
                score,
                importance
            FROM translation_grouped
            WHERE written_rep = ?1
            ORDER BY score DESC
            "#,
        )?;

        let rows = statement.query_map([word], |row| {
            Ok(TranslationGrouped {
                lexentry: value_to_optional_string(row.get_ref(0)?)?,
                written_rep: row.get(1)?,
                min_sense_num: value_to_optional_i32(row.get_ref(2)?)?,
                sense_list: TranslationList::parse(
                    value_to_string(row.get_ref(3)?)?,
                ),
                trans_list: TranslationList::parse(
                    value_to_string(row.get_ref(4)?)?,
                ),
                score: value_to_f32(row.get_ref(5)?)?,
                importance: value_to_f32(row.get_ref(6)?)?,
            })
        })?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Looks up detailed translation records.
    pub fn lookup_translations(
        &self,
        word: &str,
    ) -> Result<Vec<Translation>, DatabaseError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                lexentry,
                sense_num,
                sense,
                written_rep,
                trans_list,
                score,
                is_good,
                importance
            FROM translation
            WHERE written_rep = ?1
            ORDER BY score DESC
            "#,
        )?;

        let rows = statement.query_map([word], |row| {
            Ok(Translation {
                lexentry: value_to_optional_string(row.get_ref(0)?)?,
                sense_num: value_to_optional_i32(row.get_ref(1)?)?,
                sense: value_to_optional_string(row.get_ref(2)?)?,
                written_rep: row.get(3)?,
                trans_list: TranslationList::parse(
                    value_to_string(row.get_ref(4)?)?,
                ),
                score: value_to_f32(row.get_ref(5)?)?,
                is_good: value_to_bool(row.get_ref(6)?)?,
                importance: value_to_f32(row.get_ref(7)?)?,
            })
        })?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

// -----------------------------------------------------------------------------
// SQLite value conversion
// -----------------------------------------------------------------------------

fn value_to_string(value: ValueRef<'_>) -> Result<String, rusqlite::Error> {
    match value {
        ValueRef::Null => Ok(String::new()),

        ValueRef::Text(value) => {
            Ok(String::from_utf8_lossy(value).into_owned())
        }

        ValueRef::Blob(value) => {
            String::from_utf8(value.to_vec()).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    value.len(),
                    rusqlite::types::Type::Blob,
                    Box::new(error),
                )
            })
        }

        ValueRef::Integer(value) => Ok(value.to_string()),

        ValueRef::Real(value) => Ok(value.to_string()),
    }
}

fn value_to_optional_string(
    value: ValueRef<'_>,
) -> Result<Option<String>, rusqlite::Error> {
    if matches!(value, ValueRef::Null) {
        Ok(None)
    } else {
        value_to_string(value).map(Some)
    }
}

fn value_to_f32(value: ValueRef<'_>) -> Result<f32, rusqlite::Error> {
    match value {
        ValueRef::Integer(value) => Ok(value as f32),

        ValueRef::Real(value) => Ok(value as f32),

        ValueRef::Text(value) => {
            let value = std::str::from_utf8(value).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    value.len(),
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;

            value.parse::<f32>().map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    value.len(),
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })
        }

        ValueRef::Blob(value) => {
            let value = std::str::from_utf8(value).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    value.len(),
                    rusqlite::types::Type::Blob,
                    Box::new(error),
                )
            })?;

            value.parse::<f32>().map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    value.len(),
                    rusqlite::types::Type::Blob,
                    Box::new(error),
                )
            })
        }

        ValueRef::Null => Ok(0.0),
    }
}

fn value_to_optional_i32(
    value: ValueRef<'_>,
) -> Result<Option<i32>, rusqlite::Error> {
    if matches!(value, ValueRef::Null) {
        return Ok(None);
    }

    match value {
        ValueRef::Integer(value) => Ok(Some(value as i32)),

        ValueRef::Real(value) => Ok(Some(value as i32)),

        ValueRef::Text(value) => {
            let value = std::str::from_utf8(value).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    value.len(),
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;

            Ok(value.parse::<i32>().ok())
        }

        ValueRef::Blob(value) => {
            let value = std::str::from_utf8(value).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    value.len(),
                    rusqlite::types::Type::Blob,
                    Box::new(error),
                )
            })?;

            Ok(value.parse::<i32>().ok())
        }

        ValueRef::Null => Ok(None),
    }
}

fn value_to_bool(value: ValueRef<'_>) -> Result<bool, rusqlite::Error> {
    match value {
        ValueRef::Integer(value) => Ok(value != 0),

        ValueRef::Real(value) => Ok(value != 0.0),

        ValueRef::Text(value) => {
            let value = std::str::from_utf8(value).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    value.len(),
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;

            Ok(matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true"
            ))
        }

        ValueRef::Blob(value) => {
            let value = std::str::from_utf8(value).map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    value.len(),
                    rusqlite::types::Type::Blob,
                    Box::new(error),
                )
            })?;

            Ok(matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true"
            ))
        }

        ValueRef::Null => Ok(false),
    }
}
