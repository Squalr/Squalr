use eframe::egui::{Context, TextureOptions};
use epaint::{ColorImage, TextureHandle};

// App.
static ICON_LOGO: &[u8] = include_bytes!("../../images/app/app_icon_small.png");
static ICON_CLOSE: &[u8] = include_bytes!("../../images/app/close.png");
static ICON_MINIMIZE: &[u8] = include_bytes!("../../images/app/minimize.png");
static ICON_MAXIMIZE: &[u8] = include_bytes!("../../images/app/maximize.png");

// Common.
static ICON_ADD: &[u8] = include_bytes!("../../images/app/common/add.png");
static ICON_REMOVE: &[u8] = include_bytes!("../../images/app/common/remove.png");
static ICON_CHECK_MARK: &[u8] = include_bytes!("../../images/app/common/check_mark.png");
static ICON_DELETE: &[u8] = include_bytes!("../../images/app/common/delete.png");
static ICON_EDIT: &[u8] = include_bytes!("../../images/app/common/edit.png");
static ICON_PROPERTIES: &[u8] = include_bytes!("../../images/app/common/properties.png");
static ICON_SEARCH: &[u8] = include_bytes!("../../images/app/common/search.png");

// Data Types.
static ICON_DATA_TYPE_BLUE_BLOCKS_1: &[u8] = include_bytes!("../../images/app/data_types/blue_blocks_1.png");
static ICON_DATA_TYPE_BLUE_BLOCKS_2: &[u8] = include_bytes!("../../images/app/data_types/blue_blocks_2.png");
static ICON_DATA_TYPE_BLUE_BLOCKS_4: &[u8] = include_bytes!("../../images/app/data_types/blue_blocks_4.png");
static ICON_DATA_TYPE_BLUE_BLOCKS_8: &[u8] = include_bytes!("../../images/app/data_types/blue_blocks_8.png");
static ICON_DATA_TYPE_BLUE_BLOCKS_ARRAY: &[u8] = include_bytes!("../../images/app/data_types/blue_blocks_array.png");
static ICON_DATA_TYPE_BLUE_BLOCKS_REVERSE_1: &[u8] = include_bytes!("../../images/app/data_types/blue_blocks_reverse_1.png");
static ICON_DATA_TYPE_BLUE_BLOCKS_REVERSE_2: &[u8] = include_bytes!("../../images/app/data_types/blue_blocks_reverse_2.png");
static ICON_DATA_TYPE_BLUE_BLOCKS_REVERSE_4: &[u8] = include_bytes!("../../images/app/data_types/blue_blocks_reverse_4.png");
static ICON_DATA_TYPE_BLUE_BLOCKS_REVERSE_8: &[u8] = include_bytes!("../../images/app/data_types/blue_blocks_reverse_8.png");
static ICON_DATA_TYPE_BOOL: &[u8] = include_bytes!("../../images/app/data_types/bool.png");
static ICON_DATA_TYPE_ORANGE_BLOCKS_1: &[u8] = include_bytes!("../../images/app/data_types/orange_blocks_1.png");
static ICON_DATA_TYPE_ORANGE_BLOCKS_2: &[u8] = include_bytes!("../../images/app/data_types/orange_blocks_2.png");
static ICON_DATA_TYPE_ORANGE_BLOCKS_4: &[u8] = include_bytes!("../../images/app/data_types/orange_blocks_4.png");
static ICON_DATA_TYPE_ORANGE_BLOCKS_8: &[u8] = include_bytes!("../../images/app/data_types/orange_blocks_8.png");
static ICON_DATA_TYPE_ORANGE_BLOCKS_REVERSE_1: &[u8] = include_bytes!("../../images/app/data_types/orange_blocks_reverse_1.png");
static ICON_DATA_TYPE_ORANGE_BLOCKS_REVERSE_2: &[u8] = include_bytes!("../../images/app/data_types/orange_blocks_reverse_2.png");
static ICON_DATA_TYPE_ORANGE_BLOCKS_REVERSE_4: &[u8] = include_bytes!("../../images/app/data_types/orange_blocks_reverse_4.png");
static ICON_DATA_TYPE_ORANGE_BLOCKS_REVERSE_8: &[u8] = include_bytes!("../../images/app/data_types/orange_blocks_reverse_8.png");
static ICON_DATA_TYPE_PURPLE_BLOCKS_1: &[u8] = include_bytes!("../../images/app/data_types/purple_blocks_1.png");
static ICON_DATA_TYPE_PURPLE_BLOCKS_2: &[u8] = include_bytes!("../../images/app/data_types/purple_blocks_2.png");
static ICON_DATA_TYPE_PURPLE_BLOCKS_4: &[u8] = include_bytes!("../../images/app/data_types/purple_blocks_4.png");
static ICON_DATA_TYPE_PURPLE_BLOCKS_8: &[u8] = include_bytes!("../../images/app/data_types/purple_blocks_8.png");
static ICON_DATA_TYPE_PURPLE_BLOCKS_ARRAY: &[u8] = include_bytes!("../../images/app/data_types/purple_blocks_array.png");
static ICON_DATA_TYPE_PURPLE_BLOCKS_REVERSE_1: &[u8] = include_bytes!("../../images/app/data_types/purple_blocks_reverse_1.png");
static ICON_DATA_TYPE_PURPLE_BLOCKS_REVERSE_2: &[u8] = include_bytes!("../../images/app/data_types/purple_blocks_reverse_2.png");
static ICON_DATA_TYPE_PURPLE_BLOCKS_REVERSE_4: &[u8] = include_bytes!("../../images/app/data_types/purple_blocks_reverse_4.png");
static ICON_DATA_TYPE_PURPLE_BLOCKS_REVERSE_8: &[u8] = include_bytes!("../../images/app/data_types/purple_blocks_reverse_8.png");
static ICON_DATA_TYPE_STRING: &[u8] = include_bytes!("../../images/app/data_types/string.png");
static ICON_DATA_TYPE_UNKNOWN: &[u8] = include_bytes!("../../images/app/data_types/unknown.png");

// Display Types.
static ICON_DISPLAY_TYPE_BINARY: &[u8] = include_bytes!("../../images/app/display_types/binary.png");
static ICON_DISPLAY_TYPE_DECIMAL: &[u8] = include_bytes!("../../images/app/display_types/decimal.png");
static ICON_DISPLAY_TYPE_HEXADECIMAL: &[u8] = include_bytes!("../../images/app/display_types/hexadecimal.png");
static ICON_DISPLAY_TYPE_STRING: &[u8] = include_bytes!("../../images/app/display_types/string.png");

// Projects.
static ICON_PROJECT_CPU_INSTRUCTION: &[u8] = include_bytes!("../../images/app/projects/cpu_instruction.png");
static ICON_PROJECT_MIDDLEWARE_DOLPHIN: &[u8] = include_bytes!("../../images/app/projects/middleware_dolphin.png");
static ICON_PROJECT_POINTER_TYPE: &[u8] = include_bytes!("../../images/app/projects/pointer_type.png");
static ICON_PROJECT_SCRIPT: &[u8] = include_bytes!("../../images/app/projects/script.png");

// Navigation.
static ICON_NAVIGATION_CANCEL: &[u8] = include_bytes!("../../images/navigation/cancel.png");
static ICON_NAVIGATION_CONNECT: &[u8] = include_bytes!("../../images/navigation/connect.png");
static ICON_NAVIGATION_DOUBLE_RIGHT_OVERLAPPED_ARROWS: &[u8] = include_bytes!("../../images/navigation/double_right_overlapped_arrows.png");
static ICON_NAVIGATION_DOWN_ARROW_SMALL: &[u8] = include_bytes!("../../images/navigation/down_arrow_small.png");
static ICON_NAVIGATION_DOWN_ARROWS: &[u8] = include_bytes!("../../images/navigation/down_arrows.png");
static ICON_NAVIGATION_HOME: &[u8] = include_bytes!("../../images/navigation/home.png");
static ICON_NAVIGATION_LEFT_ARROW_SMALL: &[u8] = include_bytes!("../../images/navigation/left_arrow_small.png");
static ICON_NAVIGATION_LEFT_ARROW: &[u8] = include_bytes!("../../images/navigation/left_arrow.png");
static ICON_NAVIGATION_LEFT_ARROWS: &[u8] = include_bytes!("../../images/navigation/left_arrows.png");
static ICON_NAVIGATION_REDO: &[u8] = include_bytes!("../../images/navigation/redo.png");
static ICON_NAVIGATION_REFRESH: &[u8] = include_bytes!("../../images/navigation/refresh.png");
static ICON_NAVIGATION_RIGHT_ARROW_SMALL: &[u8] = include_bytes!("../../images/navigation/right_arrow_small.png");
static ICON_NAVIGATION_RIGHT_ARROW: &[u8] = include_bytes!("../../images/navigation/right_arrow.png");
static ICON_NAVIGATION_RIGHT_ARROWS: &[u8] = include_bytes!("../../images/navigation/right_arrows.png");
static ICON_NAVIGATION_STOP: &[u8] = include_bytes!("../../images/navigation/stop.png");
static ICON_NAVIGATION_UNDO: &[u8] = include_bytes!("../../images/navigation/undo.png");
static ICON_NAVIGATION_UP_ARROW_SMALL: &[u8] = include_bytes!("../../images/navigation/up_arrow_small.png");

// Results.
static ICON_RESULT_FREEZE: &[u8] = include_bytes!("../../images/app/results/freeze.png");

// Scans.
static ICON_SCAN_NEGATION: &[u8] = include_bytes!("../../images/app/scans/negation.png");
static ICON_SCAN_COLLECT_VALUES: &[u8] = include_bytes!("../../images/app/scans/scan_collect_values.png");
static ICON_SCAN_DELTA_DECREASED_BY_X: &[u8] = include_bytes!("../../images/app/scans/scan_delta_decreased_by_x.png");
static ICON_SCAN_DELTA_DIVIDED_BY_X: &[u8] = include_bytes!("../../images/app/scans/scan_delta_divided_by_x.png");
static ICON_SCAN_DELTA_INCREASED_BY_X: &[u8] = include_bytes!("../../images/app/scans/scan_delta_increased_by_x.png");
static ICON_SCAN_DELTA_LOGICAL_AND_BY_X: &[u8] = include_bytes!("../../images/app/scans/scan_delta_logical_and_by_x.png");
static ICON_SCAN_DELTA_LOGICAL_OR_BY_X: &[u8] = include_bytes!("../../images/app/scans/scan_delta_logical_or_by_x.png");
static ICON_SCAN_DELTA_LOGICAL_XOR_BY_X: &[u8] = include_bytes!("../../images/app/scans/scan_delta_logical_xor_by_x.png");
static ICON_SCAN_DELTA_MODULO_BY_X: &[u8] = include_bytes!("../../images/app/scans/scan_delta_modulo_by_x.png");
static ICON_SCAN_DELTA_MULTIPLIED_BY_X: &[u8] = include_bytes!("../../images/app/scans/scan_delta_multiplied_by_x.png");
static ICON_SCAN_DELTA_SHIFT_LEFT_BY_X: &[u8] = include_bytes!("../../images/app/scans/scan_delta_shift_left_by_x.png");
static ICON_SCAN_DELTA_SHIFT_RIGHT_BY_X: &[u8] = include_bytes!("../../images/app/scans/scan_delta_shift_right_by_x.png");
static ICON_SCAN_IMMEDIATE_EQUAL: &[u8] = include_bytes!("../../images/app/scans/scan_immediate_equal.png");
static ICON_SCAN_IMMEDIATE_GREATER_THAN_OR_EQUAL: &[u8] = include_bytes!("../../images/app/scans/scan_immediate_greater_than_or_equal.png");
static ICON_SCAN_IMMEDIATE_GREATER_THAN: &[u8] = include_bytes!("../../images/app/scans/scan_immediate_greater_than.png");
static ICON_SCAN_IMMEDIATE_LESS_THAN_OR_EQUAL: &[u8] = include_bytes!("../../images/app/scans/scan_immediate_less_than_or_equal.png");
static ICON_SCAN_IMMEDIATE_LESS_THAN: &[u8] = include_bytes!("../../images/app/scans/scan_immediate_less_than.png");
static ICON_SCAN_IMMEDIATE_NOT_EQUAL: &[u8] = include_bytes!("../../images/app/scans/scan_immediate_not_equal.png");
static ICON_SCAN_NEW: &[u8] = include_bytes!("../../images/app/scans/scan_new.png");
static ICON_SCAN_RELATIVE_CHANGED: &[u8] = include_bytes!("../../images/app/scans/scan_relative_changed.png");
static ICON_SCAN_RELATIVE_DECREASED: &[u8] = include_bytes!("../../images/app/scans/scan_relative_decreased.png");
static ICON_SCAN_RELATIVE_INCREASED: &[u8] = include_bytes!("../../images/app/scans/scan_relative_increased.png");
static ICON_SCAN_RELATIVE_UNCHANGED: &[u8] = include_bytes!("../../images/app/scans/scan_relative_unchanged.png");

// File system.
static ICON_FILE_SYSTEM_MERGE_FOLDERS: &[u8] = include_bytes!("../../images/file_system/merge_folders.png");
static ICON_FILE_SYSTEM_OPEN_FOLDER: &[u8] = include_bytes!("../../images/file_system/open_folder.png");
static ICON_FILE_SYSTEM_BROWSE_FOLDER: &[u8] = include_bytes!("../../images/file_system/browse_folder.png");
static ICON_FILE_SYSTEM_SAVE: &[u8] = include_bytes!("../../images/file_system/save.png");

pub struct IconLibrary {
    // App.
    pub icon_handle_logo: TextureHandle,
    pub icon_handle_close: TextureHandle,
    pub icon_handle_minimize: TextureHandle,
    pub icon_handle_maximize: TextureHandle,

    // Common.
    pub icon_handle_common_add: TextureHandle,
    pub icon_handle_common_remove: TextureHandle,
    pub icon_handle_common_check_mark: TextureHandle,
    pub icon_handle_common_delete: TextureHandle,
    pub icon_handle_common_edit: TextureHandle,
    pub icon_handle_common_properties: TextureHandle,
    pub icon_handle_common_search: TextureHandle,

    // Data Types.
    pub icon_handle_data_type_blue_blocks_1: TextureHandle,
    pub icon_handle_data_type_blue_blocks_2: TextureHandle,
    pub icon_handle_data_type_blue_blocks_4: TextureHandle,
    pub icon_handle_data_type_blue_blocks_8: TextureHandle,
    pub icon_handle_data_type_blue_blocks_array: TextureHandle,
    pub icon_handle_data_type_blue_blocks_reverse_1: TextureHandle,
    pub icon_handle_data_type_blue_blocks_reverse_2: TextureHandle,
    pub icon_handle_data_type_blue_blocks_reverse_4: TextureHandle,
    pub icon_handle_data_type_blue_blocks_reverse_8: TextureHandle,
    pub icon_handle_data_type_bool: TextureHandle,
    pub icon_handle_data_type_orange_blocks_1: TextureHandle,
    pub icon_handle_data_type_orange_blocks_2: TextureHandle,
    pub icon_handle_data_type_orange_blocks_4: TextureHandle,
    pub icon_handle_data_type_orange_blocks_8: TextureHandle,
    pub icon_handle_data_type_orange_blocks_reverse_1: TextureHandle,
    pub icon_handle_data_type_orange_blocks_reverse_2: TextureHandle,
    pub icon_handle_data_type_orange_blocks_reverse_4: TextureHandle,
    pub icon_handle_data_type_orange_blocks_reverse_8: TextureHandle,
    pub icon_handle_data_type_purple_blocks_1: TextureHandle,
    pub icon_handle_data_type_purple_blocks_2: TextureHandle,
    pub icon_handle_data_type_purple_blocks_4: TextureHandle,
    pub icon_handle_data_type_purple_blocks_8: TextureHandle,
    pub icon_handle_data_type_purple_blocks_array: TextureHandle,
    pub icon_handle_data_type_purple_blocks_reverse_1: TextureHandle,
    pub icon_handle_data_type_purple_blocks_reverse_2: TextureHandle,
    pub icon_handle_data_type_purple_blocks_reverse_4: TextureHandle,
    pub icon_handle_data_type_purple_blocks_reverse_8: TextureHandle,
    pub icon_handle_data_type_string: TextureHandle,
    pub icon_handle_data_type_unknown: TextureHandle,

    // Display Types.
    pub icon_handle_display_type_binary: TextureHandle,
    pub icon_handle_display_type_decimal: TextureHandle,
    pub icon_handle_display_type_hexadecimal: TextureHandle,
    pub icon_handle_display_type_string: TextureHandle,

    // Projects.
    pub icon_handle_project_cpu_instruction: TextureHandle,
    pub icon_handle_project_middleware_dolphin: TextureHandle,
    pub icon_handle_project_pointer_type: TextureHandle,
    pub icon_handle_project_script: TextureHandle,

    // Navigation.
    pub icon_handle_navigation_cancel: TextureHandle,
    pub icon_handle_navigation_connect: TextureHandle,
    pub icon_handle_navigation_double_right_overlapped_arrows: TextureHandle,
    pub icon_handle_navigation_down_arrow_small: TextureHandle,
    pub icon_handle_navigation_down_arrows: TextureHandle,
    pub icon_handle_navigation_home: TextureHandle,
    pub icon_handle_navigation_left_arrow_small: TextureHandle,
    pub icon_handle_navigation_left_arrow: TextureHandle,
    pub icon_handle_navigation_left_arrows: TextureHandle,
    pub icon_handle_navigation_redo: TextureHandle,
    pub icon_handle_navigation_refresh: TextureHandle,
    pub icon_handle_navigation_right_arrow_small: TextureHandle,
    pub icon_handle_navigation_right_arrow: TextureHandle,
    pub icon_handle_navigation_right_arrows: TextureHandle,
    pub icon_handle_navigation_stop: TextureHandle,
    pub icon_handle_navigation_undo: TextureHandle,
    pub icon_handle_navigation_up_arrow_small: TextureHandle,

    // Results.
    pub icon_handle_results_freeze: TextureHandle,

    // Scans.
    pub icon_handle_scan_negation: TextureHandle,
    pub icon_handle_scan_collect_values: TextureHandle,
    pub icon_handle_scan_delta_decreased_by_x: TextureHandle,
    pub icon_handle_scan_delta_divided_by_x: TextureHandle,
    pub icon_handle_scan_delta_increased_by_x: TextureHandle,
    pub icon_handle_scan_delta_logical_and_by_x: TextureHandle,
    pub icon_handle_scan_delta_logical_or_by_x: TextureHandle,
    pub icon_handle_scan_delta_logical_xor_by_x: TextureHandle,
    pub icon_handle_scan_delta_modulo_by_x: TextureHandle,
    pub icon_handle_scan_delta_multiplied_by_x: TextureHandle,
    pub icon_handle_scan_delta_shift_left_by_x: TextureHandle,
    pub icon_handle_scan_delta_shift_right_by_x: TextureHandle,
    pub icon_handle_scan_immediate_equal: TextureHandle,
    pub icon_handle_scan_immediate_greater_than_or_equal: TextureHandle,
    pub icon_handle_scan_immediate_greater_than: TextureHandle,
    pub icon_handle_scan_immediate_less_than_or_equal: TextureHandle,
    pub icon_handle_scan_immediate_less_than: TextureHandle,
    pub icon_handle_scan_immediate_not_equal: TextureHandle,
    pub icon_handle_scan_new: TextureHandle,
    pub icon_handle_scan_relative_changed: TextureHandle,
    pub icon_handle_scan_relative_decreased: TextureHandle,
    pub icon_handle_scan_relative_increased: TextureHandle,
    pub icon_handle_scan_relative_unchanged: TextureHandle,

    // File system.
    pub icon_handle_file_system_merge_folders: TextureHandle,
    pub icon_handle_file_system_open_folder: TextureHandle,
    pub icon_handle_file_system_browse_folder: TextureHandle,
    pub icon_handle_file_system_save: TextureHandle,
}

impl IconLibrary {
    pub fn new(context: &Context) -> Self {
        // App.
        let icon_handle_logo = Self::load_icon(context, ICON_LOGO);
        let icon_handle_close = Self::load_icon(context, ICON_CLOSE);
        let icon_handle_minimize = Self::load_icon(context, ICON_MINIMIZE);
        let icon_handle_maximize = Self::load_icon(context, ICON_MAXIMIZE);

        // Common.
        let icon_handle_common_add = Self::load_icon(context, ICON_ADD);
        let icon_handle_common_remove = Self::load_icon(context, ICON_REMOVE);
        let icon_handle_common_check_mark = Self::load_icon(context, ICON_CHECK_MARK);
        let icon_handle_common_delete = Self::load_icon(context, ICON_DELETE);
        let icon_handle_common_edit = Self::load_icon(context, ICON_EDIT);
        let icon_handle_common_properties = Self::load_icon(context, ICON_PROPERTIES);
        let icon_handle_common_search = Self::load_icon(context, ICON_SEARCH);

        // Data Types.
        let icon_handle_data_type_blue_blocks_1 = Self::load_icon(context, ICON_DATA_TYPE_BLUE_BLOCKS_1);
        let icon_handle_data_type_blue_blocks_2 = Self::load_icon(context, ICON_DATA_TYPE_BLUE_BLOCKS_2);
        let icon_handle_data_type_blue_blocks_4 = Self::load_icon(context, ICON_DATA_TYPE_BLUE_BLOCKS_4);
        let icon_handle_data_type_blue_blocks_8 = Self::load_icon(context, ICON_DATA_TYPE_BLUE_BLOCKS_8);
        let icon_handle_data_type_blue_blocks_array = Self::load_icon(context, ICON_DATA_TYPE_BLUE_BLOCKS_ARRAY);
        let icon_handle_data_type_blue_blocks_reverse_1 = Self::load_icon(context, ICON_DATA_TYPE_BLUE_BLOCKS_REVERSE_1);
        let icon_handle_data_type_blue_blocks_reverse_2 = Self::load_icon(context, ICON_DATA_TYPE_BLUE_BLOCKS_REVERSE_2);
        let icon_handle_data_type_blue_blocks_reverse_4 = Self::load_icon(context, ICON_DATA_TYPE_BLUE_BLOCKS_REVERSE_4);
        let icon_handle_data_type_blue_blocks_reverse_8 = Self::load_icon(context, ICON_DATA_TYPE_BLUE_BLOCKS_REVERSE_8);
        let icon_handle_data_type_bool = Self::load_icon(context, ICON_DATA_TYPE_BOOL);
        let icon_handle_data_type_orange_blocks_1 = Self::load_icon(context, ICON_DATA_TYPE_ORANGE_BLOCKS_1);
        let icon_handle_data_type_orange_blocks_2 = Self::load_icon(context, ICON_DATA_TYPE_ORANGE_BLOCKS_2);
        let icon_handle_data_type_orange_blocks_4 = Self::load_icon(context, ICON_DATA_TYPE_ORANGE_BLOCKS_4);
        let icon_handle_data_type_orange_blocks_8 = Self::load_icon(context, ICON_DATA_TYPE_ORANGE_BLOCKS_8);
        let icon_handle_data_type_orange_blocks_reverse_1 = Self::load_icon(context, ICON_DATA_TYPE_ORANGE_BLOCKS_REVERSE_1);
        let icon_handle_data_type_orange_blocks_reverse_2 = Self::load_icon(context, ICON_DATA_TYPE_ORANGE_BLOCKS_REVERSE_2);
        let icon_handle_data_type_orange_blocks_reverse_4 = Self::load_icon(context, ICON_DATA_TYPE_ORANGE_BLOCKS_REVERSE_4);
        let icon_handle_data_type_orange_blocks_reverse_8 = Self::load_icon(context, ICON_DATA_TYPE_ORANGE_BLOCKS_REVERSE_8);
        let icon_handle_data_type_purple_blocks_1 = Self::load_icon(context, ICON_DATA_TYPE_PURPLE_BLOCKS_1);
        let icon_handle_data_type_purple_blocks_2 = Self::load_icon(context, ICON_DATA_TYPE_PURPLE_BLOCKS_2);
        let icon_handle_data_type_purple_blocks_4 = Self::load_icon(context, ICON_DATA_TYPE_PURPLE_BLOCKS_4);
        let icon_handle_data_type_purple_blocks_8 = Self::load_icon(context, ICON_DATA_TYPE_PURPLE_BLOCKS_8);
        let icon_handle_data_type_purple_blocks_array = Self::load_icon(context, ICON_DATA_TYPE_PURPLE_BLOCKS_ARRAY);
        let icon_handle_data_type_purple_blocks_reverse_1 = Self::load_icon(context, ICON_DATA_TYPE_PURPLE_BLOCKS_REVERSE_1);
        let icon_handle_data_type_purple_blocks_reverse_2 = Self::load_icon(context, ICON_DATA_TYPE_PURPLE_BLOCKS_REVERSE_2);
        let icon_handle_data_type_purple_blocks_reverse_4 = Self::load_icon(context, ICON_DATA_TYPE_PURPLE_BLOCKS_REVERSE_4);
        let icon_handle_data_type_purple_blocks_reverse_8 = Self::load_icon(context, ICON_DATA_TYPE_PURPLE_BLOCKS_REVERSE_8);
        let icon_handle_data_type_string = Self::load_icon(context, ICON_DATA_TYPE_STRING);
        let icon_handle_data_type_unknown = Self::load_icon(context, ICON_DATA_TYPE_UNKNOWN);

        // Display Types.
        let icon_handle_display_type_binary = Self::load_icon(context, ICON_DISPLAY_TYPE_BINARY);
        let icon_handle_display_type_decimal = Self::load_icon(context, ICON_DISPLAY_TYPE_DECIMAL);
        let icon_handle_display_type_hexadecimal = Self::load_icon(context, ICON_DISPLAY_TYPE_HEXADECIMAL);
        let icon_handle_display_type_string = Self::load_icon(context, ICON_DISPLAY_TYPE_STRING);

        // Projects.
        let icon_handle_project_cpu_instruction = Self::load_icon(context, ICON_PROJECT_CPU_INSTRUCTION);
        let icon_handle_project_middleware_dolphin = Self::load_icon(context, ICON_PROJECT_MIDDLEWARE_DOLPHIN);
        let icon_handle_project_pointer_type = Self::load_icon(context, ICON_PROJECT_POINTER_TYPE);
        let icon_handle_project_script = Self::load_icon(context, ICON_PROJECT_SCRIPT);

        // Navigation.
        let icon_handle_navigation_cancel = Self::load_icon(context, ICON_NAVIGATION_CANCEL);
        let icon_handle_navigation_connect = Self::load_icon(context, ICON_NAVIGATION_CONNECT);
        let icon_handle_navigation_double_right_overlapped_arrows = Self::load_icon(context, ICON_NAVIGATION_DOUBLE_RIGHT_OVERLAPPED_ARROWS);
        let icon_handle_navigation_down_arrow_small = Self::load_icon(context, ICON_NAVIGATION_DOWN_ARROW_SMALL);
        let icon_handle_navigation_down_arrows = Self::load_icon(context, ICON_NAVIGATION_DOWN_ARROWS);
        let icon_handle_navigation_home = Self::load_icon(context, ICON_NAVIGATION_HOME);
        let icon_handle_navigation_left_arrow_small = Self::load_icon(context, ICON_NAVIGATION_LEFT_ARROW_SMALL);
        let icon_handle_navigation_left_arrow = Self::load_icon(context, ICON_NAVIGATION_LEFT_ARROW);
        let icon_handle_navigation_left_arrows = Self::load_icon(context, ICON_NAVIGATION_LEFT_ARROWS);
        let icon_handle_navigation_redo = Self::load_icon(context, ICON_NAVIGATION_REDO);
        let icon_handle_navigation_refresh = Self::load_icon(context, ICON_NAVIGATION_REFRESH);
        let icon_handle_navigation_right_arrow_small = Self::load_icon(context, ICON_NAVIGATION_RIGHT_ARROW_SMALL);
        let icon_handle_navigation_right_arrow = Self::load_icon(context, ICON_NAVIGATION_RIGHT_ARROW);
        let icon_handle_navigation_right_arrows = Self::load_icon(context, ICON_NAVIGATION_RIGHT_ARROWS);
        let icon_handle_navigation_stop = Self::load_icon(context, ICON_NAVIGATION_STOP);
        let icon_handle_navigation_undo = Self::load_icon(context, ICON_NAVIGATION_UNDO);
        let icon_handle_navigation_up_arrow_small = Self::load_icon(context, ICON_NAVIGATION_UP_ARROW_SMALL);

        // Results.
        let icon_handle_results_freeze = Self::load_icon(context, ICON_RESULT_FREEZE);

        // Scans.
        let icon_handle_scan_negation = Self::load_icon(context, ICON_SCAN_NEGATION);
        let icon_handle_scan_collect_values = Self::load_icon(context, ICON_SCAN_COLLECT_VALUES);
        let icon_handle_scan_delta_decreased_by_x = Self::load_icon(context, ICON_SCAN_DELTA_DECREASED_BY_X);
        let icon_handle_scan_delta_divided_by_x = Self::load_icon(context, ICON_SCAN_DELTA_DIVIDED_BY_X);
        let icon_handle_scan_delta_increased_by_x = Self::load_icon(context, ICON_SCAN_DELTA_INCREASED_BY_X);
        let icon_handle_scan_delta_logical_and_by_x = Self::load_icon(context, ICON_SCAN_DELTA_LOGICAL_AND_BY_X);
        let icon_handle_scan_delta_logical_or_by_x = Self::load_icon(context, ICON_SCAN_DELTA_LOGICAL_OR_BY_X);
        let icon_handle_scan_delta_logical_xor_by_x = Self::load_icon(context, ICON_SCAN_DELTA_LOGICAL_XOR_BY_X);
        let icon_handle_scan_delta_modulo_by_x = Self::load_icon(context, ICON_SCAN_DELTA_MODULO_BY_X);
        let icon_handle_scan_delta_multiplied_by_x = Self::load_icon(context, ICON_SCAN_DELTA_MULTIPLIED_BY_X);
        let icon_handle_scan_delta_shift_left_by_x = Self::load_icon(context, ICON_SCAN_DELTA_SHIFT_LEFT_BY_X);
        let icon_handle_scan_delta_shift_right_by_x = Self::load_icon(context, ICON_SCAN_DELTA_SHIFT_RIGHT_BY_X);
        let icon_handle_scan_immediate_equal = Self::load_icon(context, ICON_SCAN_IMMEDIATE_EQUAL);
        let icon_handle_scan_immediate_greater_than_or_equal = Self::load_icon(context, ICON_SCAN_IMMEDIATE_GREATER_THAN_OR_EQUAL);
        let icon_handle_scan_immediate_greater_than = Self::load_icon(context, ICON_SCAN_IMMEDIATE_GREATER_THAN);
        let icon_handle_scan_immediate_less_than_or_equal = Self::load_icon(context, ICON_SCAN_IMMEDIATE_LESS_THAN_OR_EQUAL);
        let icon_handle_scan_immediate_less_than = Self::load_icon(context, ICON_SCAN_IMMEDIATE_LESS_THAN);
        let icon_handle_scan_immediate_not_equal = Self::load_icon(context, ICON_SCAN_IMMEDIATE_NOT_EQUAL);
        let icon_handle_scan_new = Self::load_icon(context, ICON_SCAN_NEW);
        let icon_handle_scan_relative_changed = Self::load_icon(context, ICON_SCAN_RELATIVE_CHANGED);
        let icon_handle_scan_relative_decreased = Self::load_icon(context, ICON_SCAN_RELATIVE_DECREASED);
        let icon_handle_scan_relative_increased = Self::load_icon(context, ICON_SCAN_RELATIVE_INCREASED);
        let icon_handle_scan_relative_unchanged = Self::load_icon(context, ICON_SCAN_RELATIVE_UNCHANGED);

        // File system.
        let icon_handle_file_system_merge_folders = Self::load_icon(context, ICON_FILE_SYSTEM_MERGE_FOLDERS);
        let icon_handle_file_system_open_folder = Self::load_icon(context, ICON_FILE_SYSTEM_OPEN_FOLDER);
        let icon_handle_file_system_browse_folder = Self::load_icon(context, ICON_FILE_SYSTEM_BROWSE_FOLDER);
        let icon_handle_file_system_save = Self::load_icon(context, ICON_FILE_SYSTEM_SAVE);

        Self {
            // App.
            icon_handle_logo,
            icon_handle_close,
            icon_handle_minimize,
            icon_handle_maximize,

            // Common.
            icon_handle_common_add,
            icon_handle_common_remove,
            icon_handle_common_check_mark,
            icon_handle_common_delete,
            icon_handle_common_edit,
            icon_handle_common_properties,
            icon_handle_common_search,

            // Data Types.
            icon_handle_data_type_blue_blocks_1,
            icon_handle_data_type_blue_blocks_2,
            icon_handle_data_type_blue_blocks_4,
            icon_handle_data_type_blue_blocks_8,
            icon_handle_data_type_blue_blocks_array,
            icon_handle_data_type_blue_blocks_reverse_1,
            icon_handle_data_type_blue_blocks_reverse_2,
            icon_handle_data_type_blue_blocks_reverse_4,
            icon_handle_data_type_blue_blocks_reverse_8,
            icon_handle_data_type_bool,
            icon_handle_data_type_orange_blocks_1,
            icon_handle_data_type_orange_blocks_2,
            icon_handle_data_type_orange_blocks_4,
            icon_handle_data_type_orange_blocks_8,
            icon_handle_data_type_orange_blocks_reverse_1,
            icon_handle_data_type_orange_blocks_reverse_2,
            icon_handle_data_type_orange_blocks_reverse_4,
            icon_handle_data_type_orange_blocks_reverse_8,
            icon_handle_data_type_purple_blocks_1,
            icon_handle_data_type_purple_blocks_2,
            icon_handle_data_type_purple_blocks_4,
            icon_handle_data_type_purple_blocks_8,
            icon_handle_data_type_purple_blocks_array,
            icon_handle_data_type_purple_blocks_reverse_1,
            icon_handle_data_type_purple_blocks_reverse_2,
            icon_handle_data_type_purple_blocks_reverse_4,
            icon_handle_data_type_purple_blocks_reverse_8,
            icon_handle_data_type_string,
            icon_handle_data_type_unknown,

            // Display Types.
            icon_handle_display_type_binary,
            icon_handle_display_type_decimal,
            icon_handle_display_type_hexadecimal,
            icon_handle_display_type_string,

            // Projects.
            icon_handle_project_cpu_instruction,
            icon_handle_project_middleware_dolphin,
            icon_handle_project_pointer_type,
            icon_handle_project_script,

            // Navigation.
            icon_handle_navigation_cancel,
            icon_handle_navigation_connect,
            icon_handle_navigation_double_right_overlapped_arrows,
            icon_handle_navigation_down_arrow_small,
            icon_handle_navigation_down_arrows,
            icon_handle_navigation_home,
            icon_handle_navigation_left_arrow_small,
            icon_handle_navigation_left_arrow,
            icon_handle_navigation_left_arrows,
            icon_handle_navigation_redo,
            icon_handle_navigation_refresh,
            icon_handle_navigation_right_arrow_small,
            icon_handle_navigation_right_arrow,
            icon_handle_navigation_right_arrows,
            icon_handle_navigation_stop,
            icon_handle_navigation_undo,
            icon_handle_navigation_up_arrow_small,

            // Results.
            icon_handle_results_freeze,

            // Scans.
            icon_handle_scan_negation,
            icon_handle_scan_collect_values,
            icon_handle_scan_delta_decreased_by_x,
            icon_handle_scan_delta_divided_by_x,
            icon_handle_scan_delta_increased_by_x,
            icon_handle_scan_delta_logical_and_by_x,
            icon_handle_scan_delta_logical_or_by_x,
            icon_handle_scan_delta_logical_xor_by_x,
            icon_handle_scan_delta_modulo_by_x,
            icon_handle_scan_delta_multiplied_by_x,
            icon_handle_scan_delta_shift_left_by_x,
            icon_handle_scan_delta_shift_right_by_x,
            icon_handle_scan_immediate_equal,
            icon_handle_scan_immediate_greater_than_or_equal,
            icon_handle_scan_immediate_greater_than,
            icon_handle_scan_immediate_less_than_or_equal,
            icon_handle_scan_immediate_less_than,
            icon_handle_scan_immediate_not_equal,
            icon_handle_scan_new,
            icon_handle_scan_relative_changed,
            icon_handle_scan_relative_decreased,
            icon_handle_scan_relative_increased,
            icon_handle_scan_relative_unchanged,

            // File system.
            icon_handle_file_system_merge_folders,
            icon_handle_file_system_open_folder,
            icon_handle_file_system_browse_folder,
            icon_handle_file_system_save,
        }
    }

    fn load_icon(
        context: &Context,
        buffer: &[u8],
    ) -> TextureHandle {
        let image = image::load_from_memory(buffer).unwrap_or_default().to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let pixels = image.into_raw();
        let texture_handle = context.load_texture("", ColorImage::from_rgba_unmultiplied(size, &pixels), TextureOptions::default());

        texture_handle
    }
}
