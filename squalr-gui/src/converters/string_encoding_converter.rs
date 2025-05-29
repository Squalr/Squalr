use crate::StringEncodingView;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use squalr_engine_api::structures::data_types::built_in_types::string::string_encoding::StringEncoding;

pub struct StringEncodingConverter {}

impl StringEncodingConverter {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConvertToViewData<StringEncoding, StringEncodingView> for StringEncodingConverter {
    fn convert_collection(
        &self,
        string_encoding_list: &Vec<StringEncoding>,
    ) -> Vec<StringEncodingView> {
        string_encoding_list
            .iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        string_encoding: &StringEncoding,
    ) -> StringEncodingView {
        match string_encoding {
            StringEncoding::Utf8 => StringEncodingView::Utf8,
            StringEncoding::Utf16 => StringEncodingView::Utf16,
            StringEncoding::Utf16be => StringEncodingView::Utf16be,
            StringEncoding::Ascii => StringEncodingView::Ascii,
            StringEncoding::Big5 => StringEncodingView::Big5,
            StringEncoding::EucJp => StringEncodingView::EucJp,
            StringEncoding::EucKr => StringEncodingView::EucKr,
            StringEncoding::Gb18030_2022 => StringEncodingView::Gb180302022,
            StringEncoding::Gbk => StringEncodingView::Gbk,
            StringEncoding::Hz => StringEncodingView::Hz,
            StringEncoding::Iso2022Jp => StringEncodingView::Iso2022Jp,
            StringEncoding::Iso8859_1 => StringEncodingView::Iso88591,
            StringEncoding::Iso8859_10 => StringEncodingView::Iso885910,
            StringEncoding::Iso8859_13 => StringEncodingView::Iso885913,
            StringEncoding::Iso8859_14 => StringEncodingView::Iso885914,
            StringEncoding::Iso8859_15 => StringEncodingView::Iso885915,
            StringEncoding::Iso8859_16 => StringEncodingView::Iso885916,
            StringEncoding::Iso8859_2 => StringEncodingView::Iso88592,
            StringEncoding::Iso8859_3 => StringEncodingView::Iso88593,
            StringEncoding::Iso8859_4 => StringEncodingView::Iso88594,
            StringEncoding::Iso8859_5 => StringEncodingView::Iso88595,
            StringEncoding::Iso8859_6 => StringEncodingView::Iso88596,
            StringEncoding::Iso8859_7 => StringEncodingView::Iso88597,
            StringEncoding::Iso8859_8 => StringEncodingView::Iso88598,
            StringEncoding::Iso8859_8I => StringEncodingView::Iso88598i,
            StringEncoding::Koi8R => StringEncodingView::Koi8R,
            StringEncoding::Koi8U => StringEncodingView::Koi8U,
            StringEncoding::Macintosh => StringEncodingView::Macintosh,
            StringEncoding::MacCyrillic => StringEncodingView::MacCyrillic,
            StringEncoding::Replacement => StringEncodingView::Replacement,
            StringEncoding::ShiftJis => StringEncodingView::ShiftJis,
            StringEncoding::Windows1250 => StringEncodingView::Windows1250,
            StringEncoding::Windows1251 => StringEncodingView::Windows1251,
            StringEncoding::Windows1252 => StringEncodingView::Windows1252,
            StringEncoding::Windows1253 => StringEncodingView::Windows1253,
            StringEncoding::Windows1254 => StringEncodingView::Windows1254,
            StringEncoding::Windows1255 => StringEncodingView::Windows1255,
            StringEncoding::Windows1256 => StringEncodingView::Windows1256,
            StringEncoding::Windows1257 => StringEncodingView::Windows1257,
            StringEncoding::Windows1258 => StringEncodingView::Windows1258,
            StringEncoding::Windows874 => StringEncodingView::Windows874,
            StringEncoding::XMacCyrillic => StringEncodingView::XMacCyrillic,
            StringEncoding::XUserDefined => StringEncodingView::XUserDefined,
        }
    }
}

impl ConvertFromViewData<StringEncoding, StringEncodingView> for StringEncodingConverter {
    fn convert_from_view_data(
        &self,
        view: &StringEncodingView,
    ) -> StringEncoding {
        match view {
            StringEncodingView::Utf8 => StringEncoding::Utf8,
            StringEncodingView::Utf16 => StringEncoding::Utf16,
            StringEncodingView::Utf16be => StringEncoding::Utf16be,
            StringEncodingView::Ascii => StringEncoding::Ascii,
            StringEncodingView::Big5 => StringEncoding::Big5,
            StringEncodingView::EucJp => StringEncoding::EucJp,
            StringEncodingView::EucKr => StringEncoding::EucKr,
            StringEncodingView::Gb180302022 => StringEncoding::Gb18030_2022,
            StringEncodingView::Gbk => StringEncoding::Gbk,
            StringEncodingView::Hz => StringEncoding::Hz,
            StringEncodingView::Iso2022Jp => StringEncoding::Iso2022Jp,
            StringEncodingView::Iso88591 => StringEncoding::Iso8859_1,
            StringEncodingView::Iso885910 => StringEncoding::Iso8859_10,
            StringEncodingView::Iso885913 => StringEncoding::Iso8859_13,
            StringEncodingView::Iso885914 => StringEncoding::Iso8859_14,
            StringEncodingView::Iso885915 => StringEncoding::Iso8859_15,
            StringEncodingView::Iso885916 => StringEncoding::Iso8859_16,
            StringEncodingView::Iso88592 => StringEncoding::Iso8859_2,
            StringEncodingView::Iso88593 => StringEncoding::Iso8859_3,
            StringEncodingView::Iso88594 => StringEncoding::Iso8859_4,
            StringEncodingView::Iso88595 => StringEncoding::Iso8859_5,
            StringEncodingView::Iso88596 => StringEncoding::Iso8859_6,
            StringEncodingView::Iso88597 => StringEncoding::Iso8859_7,
            StringEncodingView::Iso88598 => StringEncoding::Iso8859_8,
            StringEncodingView::Iso88598i => StringEncoding::Iso8859_8I,
            StringEncodingView::Koi8R => StringEncoding::Koi8R,
            StringEncodingView::Koi8U => StringEncoding::Koi8U,
            StringEncodingView::Macintosh => StringEncoding::Macintosh,
            StringEncodingView::MacCyrillic => StringEncoding::MacCyrillic,
            StringEncodingView::Replacement => StringEncoding::Replacement,
            StringEncodingView::ShiftJis => StringEncoding::ShiftJis,
            StringEncodingView::Windows1250 => StringEncoding::Windows1250,
            StringEncodingView::Windows1251 => StringEncoding::Windows1251,
            StringEncodingView::Windows1252 => StringEncoding::Windows1252,
            StringEncodingView::Windows1253 => StringEncoding::Windows1253,
            StringEncodingView::Windows1254 => StringEncoding::Windows1254,
            StringEncodingView::Windows1255 => StringEncoding::Windows1255,
            StringEncodingView::Windows1256 => StringEncoding::Windows1256,
            StringEncodingView::Windows1257 => StringEncoding::Windows1257,
            StringEncodingView::Windows1258 => StringEncoding::Windows1258,
            StringEncodingView::Windows874 => StringEncoding::Windows874,
            StringEncodingView::XMacCyrillic => StringEncoding::XMacCyrillic,
            StringEncodingView::XUserDefined => StringEncoding::XUserDefined,
        }
    }
}
