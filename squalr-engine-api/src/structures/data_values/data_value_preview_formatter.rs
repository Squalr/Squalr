use super::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType};

/// Options for compact anonymous value previews.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DataValuePreviewFormatOptions {
    max_array_preview_display_element_count: usize,
    max_array_preview_character_count: usize,
    max_string_preview_character_count: usize,
}

impl DataValuePreviewFormatOptions {
    /// Creates preview formatting options for array and fallback string truncation.
    pub const fn new(
        max_array_preview_display_element_count: usize,
        max_array_preview_character_count: usize,
        max_string_preview_character_count: usize,
    ) -> Self {
        Self {
            max_array_preview_display_element_count,
            max_array_preview_character_count,
            max_string_preview_character_count,
        }
    }
}

/// Formats typed value strings for compact previews.
pub struct DataValuePreviewFormatter;

impl DataValuePreviewFormatter {
    pub const DEFAULT_MAX_ARRAY_PREVIEW_ELEMENT_COUNT: u64 = 4;

    /// Returns the same container, except large fixed arrays are limited for preview reads.
    pub fn limit_array_container_type(container_type: ContainerType) -> ContainerType {
        match container_type {
            ContainerType::ArrayFixed(length) if length > Self::DEFAULT_MAX_ARRAY_PREVIEW_ELEMENT_COUNT => {
                ContainerType::ArrayFixed(Self::DEFAULT_MAX_ARRAY_PREVIEW_ELEMENT_COUNT)
            }
            container_type => container_type,
        }
    }

    /// Returns whether a value read for this container should be treated as a truncated preview.
    pub fn array_preview_was_truncated(container_type: ContainerType) -> bool {
        matches!(
            container_type,
            ContainerType::ArrayFixed(length) if length > Self::DEFAULT_MAX_ARRAY_PREVIEW_ELEMENT_COUNT
        )
    }

    /// Formats an anonymized value for a compact preview row.
    pub fn format_anonymous_value_preview(
        anonymous_value_string: &AnonymousValueString,
        contextual_container_type: ContainerType,
        preview_was_truncated: bool,
        format_options: DataValuePreviewFormatOptions,
    ) -> String {
        let effective_container_type = if matches!(anonymous_value_string.get_container_type(), ContainerType::Array | ContainerType::ArrayFixed(_)) {
            anonymous_value_string.get_container_type()
        } else {
            contextual_container_type
        };
        let display_value = anonymous_value_string.get_anonymous_value_string();
        let anonymous_value_string_format = anonymous_value_string.get_anonymous_value_string_format();

        if anonymous_value_string_format == AnonymousValueStringFormat::String {
            return Self::truncate_string_preview_value(display_value, format_options);
        }

        if matches!(effective_container_type, ContainerType::Array | ContainerType::ArrayFixed(_)) && !display_value.is_empty() {
            let preview_value = if preview_was_truncated {
                Self::append_preview_ellipsis(display_value, format_options)
            } else {
                Self::truncate_preview_value(display_value, format_options)
            };

            format!("[{}]", preview_value)
        } else {
            display_value.to_string()
        }
    }

    fn truncate_string_preview_value(
        display_value: &str,
        format_options: DataValuePreviewFormatOptions,
    ) -> String {
        let display_value_character_count = display_value.chars().count();

        if display_value_character_count <= format_options.max_string_preview_character_count {
            return display_value.to_string();
        }

        let truncated_prefix: String = display_value
            .chars()
            .take(format_options.max_string_preview_character_count)
            .collect::<String>()
            .trim_end_matches(char::is_whitespace)
            .to_string();

        format!("{}...", truncated_prefix)
    }

    fn append_preview_ellipsis(
        display_value: &str,
        format_options: DataValuePreviewFormatOptions,
    ) -> String {
        if let Some(truncated_array_preview) = Self::format_preview_from_elements(display_value, true, format_options) {
            return truncated_array_preview;
        }

        let trimmed_display_value = display_value.trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'));

        if trimmed_display_value.is_empty() {
            String::from("...")
        } else {
            format!("{}...", trimmed_display_value)
        }
    }

    fn truncate_preview_value(
        display_value: &str,
        format_options: DataValuePreviewFormatOptions,
    ) -> String {
        if let Some(truncated_array_preview) = Self::format_preview_from_elements(display_value, false, format_options) {
            return truncated_array_preview;
        }

        let display_value_character_count = display_value.chars().count();

        if display_value_character_count <= format_options.max_array_preview_character_count {
            return display_value.to_string();
        }

        let truncated_prefix: String = display_value
            .chars()
            .take(format_options.max_array_preview_character_count)
            .collect::<String>()
            .trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'))
            .to_string();

        format!("{}...", truncated_prefix)
    }

    fn format_preview_from_elements(
        display_value: &str,
        force_ellipsis: bool,
        format_options: DataValuePreviewFormatOptions,
    ) -> Option<String> {
        let array_elements = Self::split_preview_elements(display_value);

        if array_elements.len() <= 1 {
            return None;
        }

        let visible_element_count = array_elements
            .len()
            .min(format_options.max_array_preview_display_element_count);
        let mut preview_elements = array_elements
            .iter()
            .take(visible_element_count)
            .map(|array_element| (*array_element).to_string())
            .collect::<Vec<_>>();
        let has_hidden_elements = force_ellipsis || array_elements.len() > visible_element_count;

        if has_hidden_elements {
            preview_elements.push(String::from("..."));
        }

        Some(preview_elements.join(", "))
    }

    fn split_preview_elements(display_value: &str) -> Vec<&str> {
        display_value
            .split([',', ';'])
            .map(str::trim)
            .filter(|array_element| !array_element.is_empty())
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::{DataValuePreviewFormatOptions, DataValuePreviewFormatter};
    use crate::structures::data_values::{
        anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
    };

    const COMPACT_FORMAT_OPTIONS: DataValuePreviewFormatOptions = DataValuePreviewFormatOptions::new(3, 24, 48);
    const WIDE_FORMAT_OPTIONS: DataValuePreviewFormatOptions = DataValuePreviewFormatOptions::new(4, 96, 96);

    #[test]
    fn scalar_preview_uses_display_value_directly() {
        let anonymous_value_string = AnonymousValueString::new(String::from("123"), AnonymousValueStringFormat::Decimal, ContainerType::None);

        assert_eq!(
            DataValuePreviewFormatter::format_anonymous_value_preview(&anonymous_value_string, ContainerType::None, false, COMPACT_FORMAT_OPTIONS),
            "123"
        );
    }

    #[test]
    fn compact_array_preview_wraps_and_limits_visible_elements() {
        let anonymous_value_string = AnonymousValueString::new(String::from("1, 2, 3, 4"), AnonymousValueStringFormat::Decimal, ContainerType::ArrayFixed(4));

        assert_eq!(
            DataValuePreviewFormatter::format_anonymous_value_preview(&anonymous_value_string, ContainerType::ArrayFixed(4), false, COMPACT_FORMAT_OPTIONS),
            "[1, 2, 3, ...]"
        );
    }

    #[test]
    fn wide_array_preview_can_show_more_visible_elements() {
        let anonymous_value_string = AnonymousValueString::new(String::from("1, 2, 3, 4"), AnonymousValueStringFormat::Decimal, ContainerType::ArrayFixed(4));

        assert_eq!(
            DataValuePreviewFormatter::format_anonymous_value_preview(&anonymous_value_string, ContainerType::ArrayFixed(4), false, WIDE_FORMAT_OPTIONS),
            "[1, 2, 3, 4]"
        );
    }

    #[test]
    fn truncated_context_appends_ellipsis_for_short_array_preview() {
        let anonymous_value_string = AnonymousValueString::new(String::from("1, 2"), AnonymousValueStringFormat::Decimal, ContainerType::ArrayFixed(2));

        assert_eq!(
            DataValuePreviewFormatter::format_anonymous_value_preview(&anonymous_value_string, ContainerType::ArrayFixed(8), true, COMPACT_FORMAT_OPTIONS),
            "[1, 2, ...]"
        );
    }

    #[test]
    fn fixed_array_container_is_limited_for_preview_reads() {
        assert_eq!(
            DataValuePreviewFormatter::limit_array_container_type(ContainerType::ArrayFixed(8)),
            ContainerType::ArrayFixed(DataValuePreviewFormatter::DEFAULT_MAX_ARRAY_PREVIEW_ELEMENT_COUNT)
        );
    }

    #[test]
    fn string_preview_does_not_use_array_brackets_for_fixed_buffers() {
        let anonymous_value_string = AnonymousValueString::new(
            String::from("/System/Library/Frameworks"),
            AnonymousValueStringFormat::String,
            ContainerType::ArrayFixed(16),
        );

        assert_eq!(
            DataValuePreviewFormatter::format_anonymous_value_preview(&anonymous_value_string, ContainerType::ArrayFixed(16), false, COMPACT_FORMAT_OPTIONS),
            "/System/Library/Frameworks"
        );
    }

    #[test]
    fn string_preview_uses_wider_string_truncation_budget() {
        let anonymous_value_string = AnonymousValueString::new(
            String::from("/System/Library/Frameworks/AppKit.framework/Versions/C/AppKit"),
            AnonymousValueStringFormat::String,
            ContainerType::ArrayFixed(64),
        );

        assert_eq!(
            DataValuePreviewFormatter::format_anonymous_value_preview(&anonymous_value_string, ContainerType::ArrayFixed(64), false, COMPACT_FORMAT_OPTIONS),
            "/System/Library/Frameworks/AppKit.framework/Vers..."
        );
    }
}
