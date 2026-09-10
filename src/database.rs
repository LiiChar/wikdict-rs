use rusqlite::{types::ValueRef, Connection};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("invalid UTF-8")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// The value of a WikDict field without making assumptions about SQLite's ANY/BLOB columns.
#[derive(Debug, Clone, PartialEq)]
pub enum WikDictValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct SimpleTranslation {
    pub written_rep: String,
    pub trans_list: WikDictValue,
    pub max_score: WikDictValue,
    pub rel_importance: WikDictValue,
}
#[derive(Debug, Clone)]
pub struct TranslationGrouped {
    pub lexentry: WikDictValue,
    pub written_rep: String,
    pub min_sense_num: WikDictValue,
    pub sense_list: WikDictValue,
    pub trans_list: WikDictValue,
    pub score: WikDictValue,
    pub importance: WikDictValue,
}
#[derive(Debug, Clone)]
pub struct Translation {
    pub lexentry: WikDictValue,
    pub sense_num: WikDictValue,
    pub sense: WikDictValue,
    pub written_rep: String,
    pub trans_list: WikDictValue,
    pub score: WikDictValue,
    pub is_good: WikDictValue,
    pub importance: WikDictValue,
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
            conn: Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?,
        })
    }
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn lookup_simple(&self, word: &str) -> Result<Vec<SimpleTranslation>, DatabaseError> {
        let mut s=self.conn.prepare("SELECT written_rep, trans_list, max_score, rel_importance FROM simple_translation WHERE written_rep = ?1 ORDER BY max_score DESC")?;
        let rows = s.query_map([word], |r| {
            Ok(SimpleTranslation {
                written_rep: r.get(0)?,
                trans_list: value(r.get_ref(1)?),
                max_score: value(r.get_ref(2)?),
                rel_importance: value(r.get_ref(3)?),
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
    pub fn search_simple(
        &self,
        prefix: &str,
        limit: u32,
    ) -> Result<Vec<SimpleTranslation>, DatabaseError> {
        let pattern = format!("{prefix}%");
        let mut s=self.conn.prepare("SELECT written_rep, trans_list, max_score, rel_importance FROM simple_translation WHERE written_rep LIKE ?1 ORDER BY max_score DESC LIMIT ?2")?;
        let rows = s.query_map(rusqlite::params![pattern, limit], |r| {
            Ok(SimpleTranslation {
                written_rep: r.get(0)?,
                trans_list: value(r.get_ref(1)?),
                max_score: value(r.get_ref(2)?),
                rel_importance: value(r.get_ref(3)?),
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
    pub fn lookup_grouped(&self, word: &str) -> Result<Vec<TranslationGrouped>, DatabaseError> {
        let mut s=self.conn.prepare("SELECT lexentry,written_rep,min_sense_num,sense_list,trans_list,score,importance FROM translation_grouped WHERE written_rep = ?1 ORDER BY score DESC")?;
        let rows = s.query_map([word], |r| {
            Ok(TranslationGrouped {
                lexentry: value(r.get_ref(0)?),
                written_rep: r.get(1)?,
                min_sense_num: value(r.get_ref(2)?),
                sense_list: value(r.get_ref(3)?),
                trans_list: value(r.get_ref(4)?),
                score: value(r.get_ref(5)?),
                importance: value(r.get_ref(6)?),
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
    pub fn lookup_translations(&self, word: &str) -> Result<Vec<Translation>, DatabaseError> {
        let mut s=self.conn.prepare("SELECT lexentry,sense_num,sense,written_rep,trans_list,score,is_good,importance FROM translation WHERE written_rep = ?1 ORDER BY score DESC")?;
        let rows = s.query_map([word], |r| {
            Ok(Translation {
                lexentry: value(r.get_ref(0)?),
                sense_num: value(r.get_ref(1)?),
                sense: value(r.get_ref(2)?),
                written_rep: r.get(3)?,
                trans_list: value(r.get_ref(4)?),
                score: value(r.get_ref(5)?),
                is_good: value(r.get_ref(6)?),
                importance: value(r.get_ref(7)?),
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
}

fn value(v: ValueRef<'_>) -> WikDictValue {
    match v {
        ValueRef::Null => WikDictValue::Null,
        ValueRef::Integer(x) => WikDictValue::Integer(x),
        ValueRef::Real(x) => WikDictValue::Real(x),
        ValueRef::Text(x) => WikDictValue::Text(String::from_utf8_lossy(x).into_owned()),
        ValueRef::Blob(x) => WikDictValue::Blob(x.to_vec()),
    }
}
