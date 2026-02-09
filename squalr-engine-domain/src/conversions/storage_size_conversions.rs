pub struct StorageSizeConversions {}

impl StorageSizeConversions {
    /// Converts a given value into a metric information storage size (ie KB, MB, GB, TB, etc.).
    pub fn value_to_metric_size(value: u128) -> String {
        // Note: u128 runs out in the Quettabytes, which uh, is larger than we should ever need in any context.
        let suffix = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB", "RB", "QB"];

        if value == 0 {
            return "0B".to_string();
        }

        let place = match value {
            value if value >= 1_000_000_000_000_000_000_000_000_000_000 => 10, // QB
            value if value >= 1_000_000_000_000_000_000_000_000_000 => 9,      // RB
            value if value >= 1_000_000_000_000_000_000_000_000 => 8,          // YB
            value if value >= 1_000_000_000_000_000_000_000 => 7,              // ZB
            value if value >= 1_000_000_000_000_000_000 => 6,                  // EB
            value if value >= 1_000_000_000_000_000 => 5,                      // PB
            value if value >= 1_000_000_000_000 => 4,                          // TB
            value if value >= 1_000_000_000 => 3,                              // GB
            value if value >= 1_000_000 => 2,                                  // MB
            value if value >= 1_000 => 1,                                      // KB
            _value => 0,                                                       // B
        };

        let base: u128 = 1000;
        let scale = base.pow(place as u32);
        let scaled = value * 10 / scale;
        let remainder = value * 10 % scale;
        let rounded = if remainder * 2 >= scale { scaled + 1 } else { scaled };

        format!("{}.{}{}", rounded / 10, rounded % 10, suffix[place])
    }

    /// Converts a given value into a binary information storage size (KiB, MiB, GiB, TiB, etc.).
    pub fn value_to_binary_size(value: u128) -> String {
        let suffix = [
            "B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB", "RiB", "QiB",
        ];

        if value == 0 {
            return "0B".to_string();
        }

        let place = match value {
            value if value >= 1u128 << 100 => 10, // QiB
            value if value >= 1u128 << 90 => 9,   // RiB
            value if value >= 1u128 << 80 => 8,   // YiB
            value if value >= 1u128 << 70 => 7,   // ZiB
            value if value >= 1u128 << 60 => 6,   // EiB
            value if value >= 1u128 << 50 => 5,   // PiB
            value if value >= 1u128 << 40 => 4,   // TiB
            value if value >= 1u128 << 30 => 3,   // GiB
            value if value >= 1u128 << 20 => 2,   // MiB
            value if value >= 1u128 << 10 => 1,   // KiB
            _value => 0,                          // B
        };

        let base: u128 = 1024;
        let scale = base.pow(place as u32);
        let scaled = value * 10 / scale;
        let remainder = value * 10 % scale;
        let rounded = if remainder * 2 >= scale { scaled + 1 } else { scaled };

        format!("{}.{}{}", rounded / 10, rounded % 10, suffix[place])
    }
}
