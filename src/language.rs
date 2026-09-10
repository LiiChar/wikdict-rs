use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Bg,
    Ca,
    Cs,
    Da,
    De,
    El,
    En,
    Es,
    Fi,
    Fr,
    Ga,
    Id,
    It,
    Ja,
    Ku,
    La,
    Lt,
    Mg,
    Nl,
    No,
    Pl,
    Pt,
    Ru,
    Sv,
    Tr,
    Zh,
}

impl Language {
    pub const ALL: [Self; 26] = [
        Self::Bg,
        Self::Ca,
        Self::Cs,
        Self::Da,
        Self::De,
        Self::El,
        Self::En,
        Self::Es,
        Self::Fi,
        Self::Fr,
        Self::Ga,
        Self::Id,
        Self::It,
        Self::Ja,
        Self::Ku,
        Self::La,
        Self::Lt,
        Self::Mg,
        Self::Nl,
        Self::No,
        Self::Pl,
        Self::Pt,
        Self::Ru,
        Self::Sv,
        Self::Tr,
        Self::Zh,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Bg => "bg",
            Self::Ca => "ca",
            Self::Cs => "cs",
            Self::Da => "da",
            Self::De => "de",
            Self::El => "el",
            Self::En => "en",
            Self::Es => "es",
            Self::Fi => "fi",
            Self::Fr => "fr",
            Self::Ga => "ga",
            Self::Id => "id",
            Self::It => "it",
            Self::Ja => "ja",
            Self::Ku => "ku",
            Self::La => "la",
            Self::Lt => "lt",
            Self::Mg => "mg",
            Self::Nl => "nl",
            Self::No => "no",
            Self::Pl => "pl",
            Self::Pt => "pt",
            Self::Ru => "ru",
            Self::Sv => "sv",
            Self::Tr => "tr",
            Self::Zh => "zh",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl FromStr for Language {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "bg" => Ok(Self::Bg),
            "ca" => Ok(Self::Ca),
            "cs" => Ok(Self::Cs),
            "da" => Ok(Self::Da),
            "de" => Ok(Self::De),
            "el" => Ok(Self::El),
            "en" => Ok(Self::En),
            "es" => Ok(Self::Es),
            "fi" => Ok(Self::Fi),
            "fr" => Ok(Self::Fr),
            "ga" => Ok(Self::Ga),
            "id" => Ok(Self::Id),
            "it" => Ok(Self::It),
            "ja" => Ok(Self::Ja),
            "ku" => Ok(Self::Ku),
            "la" => Ok(Self::La),
            "lt" => Ok(Self::Lt),
            "mg" => Ok(Self::Mg),
            "nl" => Ok(Self::Nl),
            "no" => Ok(Self::No),
            "pl" => Ok(Self::Pl),
            "pt" => Ok(Self::Pt),
            "ru" => Ok(Self::Ru),
            "sv" => Ok(Self::Sv),
            "tr" => Ok(Self::Tr),
            "zh" => Ok(Self::Zh),
            _ => Err(format!("unsupported WikDict language: {value}")),
        }
    }
}
