use squalr_engine_api::structures::data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType};

/// Formats typed value strings for compact row previews.
pub struct DataValuePreviewFormatter;

impl DataValuePreviewFormatter {
    pub const MAX_ARRAY_PREVIEW_ELEMENT_COUNT: u64 = 4;
    const MAX_ARRAY_PREVIEW_DISPLAY_ELEMENT_COUNT: usize = 3;
    const MAX_ARRAY_PREVIEW_CHARACTER_COUNT: usize = 24;

    /// Returns whether a value read for this container should be treated as a truncated preview.
    pub fn array_preview_was_truncated(container_type: ContainerType) -> bool {
        matches!(
            container_type,
            ContainerType::ArrayFixed(length) if length > Self::MAX_ARRAY_PREVIEW_ELEMENT_COUNT
        )
    }

    /// Formats an anonymized value for a compact preview row.
    pub fn format_anonymous_value_preview(
        anonymous_value_string: &AnonymousValueString,
        contextual_container_type: ContainerType,
        preview_was_truncated: bool,
    ) -> String {
        let effective_container_type = if matches!(anonymous_value_string.get_container_type(), ContainerType::Array | ContainerType::ArrayFixed(_)) {
            anonymous_value_string.get_container_type()
        } else {
            contextual_container_type
        };
        let display_value = anonymous_value_string.get_anonymous_value_string();

        if matches!(effective_container_type, ContainerType::Array | ContainerType::ArrayFixed(_)) && !display_value.is_empty() {
            let preview_value = if preview_was_truncated {
                Self::append_preview_ellipsis(display_value)
            } else {
                Self::truncate_preview_value(display_value)
            };

            format!("[{}]", preview_value)
        } else {
            display_value.to_string()
        }
    }

    fn append_preview_ellipsis(display_value: &str) -> String {
        if let Some(truncated_array_preview) = Self::format_preview_from_elements(display_value, true) {
            return truncated_array_preview;
        }

        let trimmed_display_value = display_value.trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'));

        if trimmed_display_value.is_empty() {
            String::from("...")
        } else {
            format!("{}...", trimmed_display_value)
        }
    }

    fn truncate_preview_value(display_value: &str) -> String {
        if let Some(truncated_array_preview) = Self::format_preview_from_elements(display_value, false) {
            return truncated_array_preview;
        }

        let display_value_character_count = display_value.chars().count();

        if display_value_character_count <= Self::MAX_ARRAY_PREVIEW_CHARACTER_COUNT {
            return display_value.to_string();
        }

        let truncated_prefix: String = display_value
            .chars()
            .take(Self::MAX_ARRAY_PREVIEW_CHARACTER_COUNT)
            .collect::<String>()
            .trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'))
            .to_string();

        format!("{}...", truncated_prefix)
    }

    fn format_preview_from_elements(
        display_value: &str,
        force_ellipsis: bool,
    ) -> Option<String> {
        let array_elements = Self::split_preview_elements(display_value);

        if array_elements.len() <= 1 {
            return None;
        }

        let visible_element_count = array_elements
            .len()
            .min(Self::MAX_ARRAY_PREVIEW_DISPLAY_ELEMENT_COUNT);
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
    use super::DataValuePreviewFormatter;
    use squalr_engine_api::structures::data_values::{
        anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
    };

    #[test]
    fn scalar_preview_uses_display_value_directly() {
        let anonymous_value_string = AnonymousValueString::new(String::from("123"), AnonymousValueStringFormat::Decimal, ContainerType::None);

        assert_eq!(
            DataValuePreviewFormatter::format_anonymous_value_preview(&anonymous_value_string, ContainerType::None, false),
            "123"
        );
    }

    #[test]
    fn array_preview_wraps_and_limits_visible_elements() {
        let anonymous_value_string = AnonymousValueString::new(String::from("1, 2, 3, 4"), AnonymousValueStringFormat::Decimal, ContainerType::ArrayFixed(4));

        assert_eq!(
            DataValuePreviewFormatter::format_anonymous_value_preview(&anonymous_value_string, ContainerType::ArrayFixed(4), false),
            "[1, 2, 3, ...]"
        );
    }

    #[test]
    fn truncated_context_appends_ellipsis_for_short_array_preview() {
        let anonymous_value_string = AnonymousValueString::new(String::from("1, 2"), AnonymousValueStringFormat::Decimal, ContainerType::ArrayFixed(2));

        assert_eq!(
            DataValuePreviewFormatter::format_anonymous_value_preview(&anonymous_value_string, ContainerType::ArrayFixed(8), true),
            "[1, 2, ...]"
        );
    }
}
