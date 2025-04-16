use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// Represents the string encoding supported in scans.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StringEncoding {
    Utf8,
    Utf16,
    Utf16be,
    Ascii,
    Big5,
    EucJp,
    EucKr,
    Gb18030,
    Gb18030_2022,
    Iso2022Jp,
    ShiftJis,
    SingleByte,
    Iso8859_1,
    Iso8859_2,
    Iso8859_3,
    Iso8859_4,
    Iso8859_5,
    Iso8859_6,
    Iso8859_7,
    Iso8859_8,
    Iso8859_10,
    Iso8859_13,
    Iso8859_14,
    Iso8859_15,
    Iso8859_16,
    Koi8R,
    Koi8U,
    Macintosh,
    MacCyrillic,
    Windows874,
    Windows1250,
    Windows1251,
    Windows1252,
    Windows1253,
    Windows1254,
    Windows1255,
    Windows1256,
    Windows1257,
    Windows1258,
    HZ,
}

impl fmt::Display for StringEncoding {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for StringEncoding {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Utf8" => Ok(StringEncoding::Utf8),
            "Utf16" => Ok(StringEncoding::Utf16),
            "Utf16be" => Ok(StringEncoding::Utf16be),
            "Ascii" => Ok(StringEncoding::Ascii),
            "Big5" => Ok(StringEncoding::Big5),
            "EucJp" => Ok(StringEncoding::EucJp),
            "EucKr" => Ok(StringEncoding::EucKr),
            "Gb18030" => Ok(StringEncoding::Gb18030),
            "Gb18030_2022" => Ok(StringEncoding::Gb18030_2022),
            "Iso2022Jp" => Ok(StringEncoding::Iso2022Jp),
            "ShiftJis" => Ok(StringEncoding::ShiftJis),
            "SingleByte" => Ok(StringEncoding::SingleByte),
            "Iso8859_1" => Ok(StringEncoding::Iso8859_1),
            "Iso8859_2" => Ok(StringEncoding::Iso8859_2),
            "Iso8859_3" => Ok(StringEncoding::Iso8859_3),
            "Iso8859_4" => Ok(StringEncoding::Iso8859_4),
            "Iso8859_5" => Ok(StringEncoding::Iso8859_5),
            "Iso8859_6" => Ok(StringEncoding::Iso8859_6),
            "Iso8859_7" => Ok(StringEncoding::Iso8859_7),
            "Iso8859_8" => Ok(StringEncoding::Iso8859_8),
            "Iso8859_10" => Ok(StringEncoding::Iso8859_10),
            "Iso8859_13" => Ok(StringEncoding::Iso8859_13),
            "Iso8859_14" => Ok(StringEncoding::Iso8859_14),
            "Iso8859_15" => Ok(StringEncoding::Iso8859_15),
            "Iso8859_16" => Ok(StringEncoding::Iso8859_16),
            "Koi8R" => Ok(StringEncoding::Koi8R),
            "Koi8U" => Ok(StringEncoding::Koi8U),
            "Macintosh" => Ok(StringEncoding::Macintosh),
            "MacCyrillic" => Ok(StringEncoding::MacCyrillic),
            "Windows874" => Ok(StringEncoding::Windows874),
            "Windows1250" => Ok(StringEncoding::Windows1250),
            "Windows1251" => Ok(StringEncoding::Windows1251),
            "Windows1252" => Ok(StringEncoding::Windows1252),
            "Windows1253" => Ok(StringEncoding::Windows1253),
            "Windows1254" => Ok(StringEncoding::Windows1254),
            "Windows1255" => Ok(StringEncoding::Windows1255),
            "Windows1256" => Ok(StringEncoding::Windows1256),
            "Windows1257" => Ok(StringEncoding::Windows1257),
            "Windows1258" => Ok(StringEncoding::Windows1258),
            "HZ" => Ok(StringEncoding::HZ),
            _ => Err(format!("Unknown encoding: {}", s)),
        }
    }
}
