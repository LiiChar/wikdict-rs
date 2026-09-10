# wikdict

Rust library for downloading and querying WikDict SQLite dictionaries. WikDict's SQLite format is the native machine-readable format; the project documents it as the source from which its other dictionary formats are generated.

## Supported release

`2_2026-06` currently exposes 26 languages:

`bg ca cs da de el en es fi fr ga id it ja ku la lt mg nl no pl pt ru sv tr zh`

The release contains all directional pairs except same-language pairs: **650 dictionaries**.

## API

```rust
use std::str::FromStr;
use wikdict::{Dictionary, WikDictDatabase, WikDictDownloader};

let dict = Dictionary::from_str("en-ru")?;
let downloader = WikDictDownloader::new()?;
downloader.download(dict, "./data/en-ru.sqlite3").await?;

let db = WikDictDatabase::open_read_only("./data/en-ru.sqlite3")?;
let result = db.lookup_simple("hello")?;
```

`simple_translation` is used for the fast word lookup. The library deliberately keeps `ANY`/`BLOB` columns as `WikDictValue` instead of guessing their serialization. That makes the reader safe against format details not represented by SQLite's declared types.

## Versioning

`Dictionary` describes the concrete 2_2026-06 pair set. `WikDictVersion` controls the URL version used by the downloader.

## Data license

WikDict data is available under CC BY-SA; check the upstream license/attribution requirements before redistributing dictionary databases.
