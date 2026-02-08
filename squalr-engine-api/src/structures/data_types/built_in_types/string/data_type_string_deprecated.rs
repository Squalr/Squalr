use crate::conversions::conversions::Conversions;
use crate::structures::data_types::data_type_error::DataTypeError;
use crate::structures::data_types::data_type_meta_data::DataTypeMetaData;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::anonymous_value_string::{AnonymousValueString, AnonymousValueStringContainer};
use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use crate::structures::data_values::data_value_interpreter::DisplayValue;
use crate::structures::data_values::data_value_interpreters::DataValueInterpreters;
use crate::structures::memory::endian::Endian;
use crate::structures::{data_types::data_type::DataType, data_values::data_value::DataValue};
use encoding::all::{HZ, ISO_8859_1};
use encoding::{EncoderTrap, Encoding};
use encoding_rs::{
    BIG5, EUC_JP, EUC_KR, GB18030, GBK, ISO_2022_JP, ISO_8859_2, ISO_8859_3, ISO_8859_4, ISO_8859_5, ISO_8859_6, ISO_8859_7, ISO_8859_8, ISO_8859_8_I,
    ISO_8859_10, ISO_8859_13, ISO_8859_14, ISO_8859_15, ISO_8859_16, KOI8_R, KOI8_U, MACINTOSH, REPLACEMENT, SHIFT_JIS, WINDOWS_874, WINDOWS_1250,
    WINDOWS_1251, WINDOWS_1252, WINDOWS_1253, WINDOWS_1254, WINDOWS_1255, WINDOWS_1256, WINDOWS_1257, WINDOWS_1258, X_MAC_CYRILLIC, X_USER_DEFINED,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataTypeString {}

impl DataTypeString {
    pub const DATA_TYPE_ID: &str = "string";

    pub fn get_data_type_id() -> &'static str {
        Self::DATA_TYPE_ID
    }

    pub fn get_value_from_primitive(str: &str) -> DataValue {
        let value_bytes = str.as_bytes();
        DataValue::new(
            DataTypeRef::new(
                Self::get_data_type_id(),
                DataTypeMetaData::EncodedString(value_bytes.len() as u64, StringEncoding::Utf8),
            ),
            value_bytes.to_vec(),
        )
    }
}

impl DataType for DataTypeString {
    fn get_data_type_id(&self) -> &str {
        Self::get_data_type_id()
    }

    fn get_icon_id(&self) -> &str {
        Self::get_icon_id()
    }

    fn get_size_in_bytes(&self) -> u64 {
        1
    }

    fn validate_value_string(
        &self,
        anonymous_value_string: &AnonymousValueString,
    ) -> bool {
        let data_type_ref = DataTypeRef::new_from_anonymous_value_string(self.get_data_type_id(), anonymous_value_string);

        // Validating a UTF string really just boils down to "can we parse the anonymous value as a string".
        match self.deanonymize_value_string(anonymous_value_string, data_type_ref) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn deanonymize_value_string(
        &self,
        anonymous_value_string: &AnonymousValueString,
        data_type_ref: DataTypeRef,
    ) -> Result<DataValue, DataTypeError> {
        if data_type_ref.get_data_type_id() != Self::get_data_type_id() {
            return Err(DataTypeError::InvalidDataTypeRef {
                data_type_ref: data_type_ref.get_data_type_id().to_string(),
            });
        }

        match data_type_ref.get_meta_data() {
            DataTypeMetaData::EncodedString(size, string_encoding) => match anonymous_value_string.get_value() {
                AnonymousValueStringContainer::BinaryValue(value_string_utf8, container_type) => {
                    let value_bytes = Conversions::binary_to_bytes(value_string_utf8).map_err(|error: &str| DataTypeError::ParseError(error.to_string()))?;

                    return Ok(DataValue::new(data_type_ref, value_bytes));
                }
                AnonymousValueStringContainer::HexadecimalValue(value_string_utf8, container_type) => {
                    let value_bytes = Conversions::hex_to_bytes(value_string_utf8).map_err(|error: &str| DataTypeError::ParseError(error.to_string()))?;

                    return Ok(DataValue::new(data_type_ref, value_bytes));
                }
                AnonymousValueStringContainer::String(value_string_utf8, container_type) => {
                    let mut string_bytes = match string_encoding {
                        StringEncoding::Utf8 => value_string_utf8.as_bytes().to_vec(),
                        StringEncoding::Utf16 => {
                            let mut bytes = vec![];
                            for utf16 in value_string_utf8.encode_utf16() {
                                bytes.extend_from_slice(&utf16.to_le_bytes());
                            }
                            bytes
                        }
                        StringEncoding::Utf16be => {
                            let mut bytes = vec![];
                            for utf16 in value_string_utf8.encode_utf16() {
                                bytes.extend_from_slice(&utf16.to_be_bytes());
                            }
                            bytes
                        }
                        StringEncoding::Ascii => value_string_utf8.as_bytes().iter().map(|&b| b & 0x7F).collect(),
                        StringEncoding::Big5 => BIG5.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::EucJp => EUC_JP.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::EucKr => EUC_KR.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Gb18030_2022 => GB18030.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Gbk => GBK.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Hz => HZ
                            .encode(&value_string_utf8, EncoderTrap::Ignore)
                            .unwrap_or(vec![]),
                        StringEncoding::Iso2022Jp => ISO_2022_JP.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Iso8859_1 => ISO_8859_1
                            .encode(&value_string_utf8, EncoderTrap::Ignore)
                            .unwrap_or(vec![]),
                        StringEncoding::Iso8859_10 => ISO_8859_10.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Iso8859_13 => ISO_8859_13.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Iso8859_14 => ISO_8859_14.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Iso8859_15 => ISO_8859_15.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Iso8859_16 => ISO_8859_16.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Iso8859_2 => ISO_8859_2.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Iso8859_3 => ISO_8859_3.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Iso8859_4 => ISO_8859_4.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Iso8859_5 => ISO_8859_5.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Iso8859_6 => ISO_8859_6.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Iso8859_7 => ISO_8859_7.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Iso8859_8 => ISO_8859_8.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Iso8859_8I => ISO_8859_8_I.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Koi8R => KOI8_R.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Koi8U => KOI8_U.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::MacCyrillic => X_MAC_CYRILLIC.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Macintosh => MACINTOSH.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Replacement => REPLACEMENT.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::ShiftJis => SHIFT_JIS.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Windows1250 => WINDOWS_1250.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Windows1251 => WINDOWS_1251.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Windows1252 => WINDOWS_1252.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Windows1253 => WINDOWS_1253.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Windows1254 => WINDOWS_1254.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Windows1255 => WINDOWS_1255.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Windows1256 => WINDOWS_1256.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Windows1257 => WINDOWS_1257.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Windows1258 => WINDOWS_1258.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::Windows874 => WINDOWS_874.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::XMacCyrillic => X_MAC_CYRILLIC.encode(value_string_utf8).0.to_vec(),
                        StringEncoding::XUserDefined => X_USER_DEFINED.encode(value_string_utf8).0.to_vec(),
                    };

                    string_bytes.truncate(*size as usize);

                    Ok(DataValue::new(data_type_ref, string_bytes))
                }
                AnonymousValueStringContainer::ByteArray(value_bytes) => Ok(DataValue::new(data_type_ref, value_bytes.clone())),
            },
            _ => Err(DataTypeError::InvalidMetaData),
        }
    }

    fn create_data_value_interpreters(
        &self,
        value_bytes: &[u8],
        data_type_meta_data: &DataTypeMetaData,
    ) -> Result<DataValueInterpreters, DataTypeError> {
        if value_bytes.is_empty() {
            return Err(DataTypeError::NoBytes);
        }

        match data_type_meta_data {
            DataTypeMetaData::EncodedString(_size, string_encoding) => {
                let decoded_string = match string_encoding {
                    StringEncoding::Utf8 => std::str::from_utf8(value_bytes)
                        .map_err(|_err| DataTypeError::DecodingError)?
                        .to_string(),
                    StringEncoding::Utf16 => {
                        if value_bytes.len() % 2 != 0 {
                            return Err(DataTypeError::DecodingError);
                        }
                        let utf16_iter = value_bytes
                            .chunks(2)
                            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]));
                        String::from_utf16(utf16_iter.collect::<Vec<_>>().as_slice()).map_err(|_err| DataTypeError::DecodingError)?
                    }
                    StringEncoding::Utf16be => {
                        if value_bytes.len() % 2 != 0 {
                            return Err(DataTypeError::DecodingError);
                        }
                        let utf16_iter = value_bytes
                            .chunks(2)
                            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]));
                        String::from_utf16(utf16_iter.collect::<Vec<_>>().as_slice()).map_err(|_err| DataTypeError::DecodingError)?
                    }
                    StringEncoding::Ascii => value_bytes.iter().map(|&b| (b & 0x7F) as char).collect(),
                    StringEncoding::Big5 => {
                        let (cow, _, had_errors) = BIG5.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::EucJp => {
                        let (cow, _, had_errors) = EUC_JP.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::EucKr => {
                        let (cow, _, had_errors) = EUC_KR.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Gb18030_2022 => {
                        let (cow, _, had_errors) = GB18030.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Gbk => {
                        let (cow, _, had_errors) = GBK.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Hz => HZ
                        .decode(value_bytes, encoding::DecoderTrap::Ignore)
                        .map_err(|_| DataTypeError::DecodingError)?,
                    StringEncoding::Iso2022Jp => {
                        let (cow, _, had_errors) = ISO_2022_JP.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Iso8859_1 => ISO_8859_1
                        .decode(value_bytes, encoding::DecoderTrap::Ignore)
                        .map_err(|_| DataTypeError::DecodingError)?,
                    StringEncoding::Iso8859_10 => {
                        let (cow, _, had_errors) = ISO_8859_10.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Iso8859_13 => {
                        let (cow, _, had_errors) = ISO_8859_13.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Iso8859_14 => {
                        let (cow, _, had_errors) = ISO_8859_14.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Iso8859_15 => {
                        let (cow, _, had_errors) = ISO_8859_15.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Iso8859_16 => {
                        let (cow, _, had_errors) = ISO_8859_16.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Iso8859_2 => {
                        let (cow, _, had_errors) = ISO_8859_2.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Iso8859_3 => {
                        let (cow, _, had_errors) = ISO_8859_3.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Iso8859_4 => {
                        let (cow, _, had_errors) = ISO_8859_4.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Iso8859_5 => {
                        let (cow, _, had_errors) = ISO_8859_5.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Iso8859_6 => {
                        let (cow, _, had_errors) = ISO_8859_6.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Iso8859_7 => {
                        let (cow, _, had_errors) = ISO_8859_7.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Iso8859_8 => {
                        let (cow, _, had_errors) = ISO_8859_8.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Iso8859_8I => {
                        let (cow, _, had_errors) = ISO_8859_8_I.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Koi8R => {
                        let (cow, _, had_errors) = KOI8_R.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Koi8U => {
                        let (cow, _, had_errors) = KOI8_U.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::MacCyrillic => {
                        let (cow, _, had_errors) = X_MAC_CYRILLIC.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Macintosh => {
                        let (cow, _, had_errors) = MACINTOSH.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Replacement => {
                        let (cow, _, had_errors) = REPLACEMENT.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::ShiftJis => {
                        let (cow, _, had_errors) = SHIFT_JIS.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Windows1250 => {
                        let (cow, _, had_errors) = WINDOWS_1250.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Windows1251 => {
                        let (cow, _, had_errors) = WINDOWS_1251.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Windows1252 => {
                        let (cow, _, had_errors) = WINDOWS_1252.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Windows1253 => {
                        let (cow, _, had_errors) = WINDOWS_1253.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Windows1254 => {
                        let (cow, _, had_errors) = WINDOWS_1254.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Windows1255 => {
                        let (cow, _, had_errors) = WINDOWS_1255.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Windows1256 => {
                        let (cow, _, had_errors) = WINDOWS_1256.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Windows1257 => {
                        let (cow, _, had_errors) = WINDOWS_1257.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Windows1258 => {
                        let (cow, _, had_errors) = WINDOWS_1258.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::Windows874 => {
                        let (cow, _, had_errors) = WINDOWS_874.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::XMacCyrillic => {
                        let (cow, _, had_errors) = X_MAC_CYRILLIC.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                    StringEncoding::XUserDefined => {
                        let (cow, _, had_errors) = X_USER_DEFINED.decode(value_bytes);
                        if had_errors {
                            return Err(DataTypeError::DecodingError);
                        }
                        cow.to_string()
                    }
                };

                Ok(DataValueInterpreters::new(
                    vec![DisplayValue::new(
                        decoded_string,
                        AnonymousValueStringFormat::String,
                        ContainerType::None,
                    )],
                    AnonymousValueStringFormat::String(ContainerType::None),
                ))
            }
            _ => Err(DataTypeError::InvalidMetaData),
        }
    }

    fn get_supported_anonymous_value_string_formats(&self) -> Vec<AnonymousValueStringFormat> {
        vec![AnonymousValueStringFormat::String(ContainerType::None)]
    }

    fn is_floating_point(&self) -> bool {
        false
    }

    fn get_endian(&self) -> Endian {
        Endian::Little
    }

    fn get_default_value(
        &self,
        data_type_ref: DataTypeRef,
    ) -> DataValue {
        DataValue::new(data_type_ref.clone(), vec![])
    }

    fn get_default_meta_data(&self) -> DataTypeMetaData {
        DataTypeMetaData::EncodedString(1, StringEncoding::Utf8)
    }

    fn get_meta_data_for_anonymous_value_string(
        &self,
        anonymous_value_string: &AnonymousValueString,
    ) -> DataTypeMetaData {
        // When parsing meta data from an anonymous value, we don't really have any context, so we just validate it against UTF-8.
        // If the value is passes as hex, we actually just validate whether we successfully can parse the hex string and re-encode it as a string.
        let string_length = match anonymous_value_string.get_value() {
            AnonymousValueStringContainer::String(string) => string.as_bytes().len(),
            AnonymousValueStringContainer::BinaryValue(string) => Conversions::binary_to_bytes(string).unwrap_or_default().len(),
            AnonymousValueStringContainer::HexadecimalValue(string) => Conversions::hex_to_bytes(string).unwrap_or_default().len(),
        } as u64;

        DataTypeMetaData::EncodedString(string_length, StringEncoding::Utf8)
    }

    fn get_meta_data_from_string(
        &self,
        string: &str,
    ) -> Result<DataTypeMetaData, DataTypeError> {
        let parts: Vec<&str> = string.splitn(2, ';').collect();

        if parts.len() < 2 {
            return Err(DataTypeError::ParseError(
                "Invalid string data type format, expected string;{byte_count};{optional_encoding_or_utf8_default}".to_string(),
            ));
        }

        let string_size = match parts[1].trim().parse::<u64>() {
            Ok(string_size) => string_size,
            Err(error) => {
                return Err(DataTypeError::ParseError(format!("Failed to parse string size: {}", error)));
            }
        };
        let encoding_string = if parts.len() >= 2 { parts[2].trim() } else { "" };
        let encoding = encoding_string.parse().unwrap_or(StringEncoding::Utf8);

        Ok(DataTypeMetaData::EncodedString(string_size, encoding))
    }
}
