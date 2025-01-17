use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::sync::OnceLock;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub mod vectors {
    use super::*;

    pub static HAS_VECTOR_SUPPORT: OnceLock<bool> = OnceLock::new();
    pub static HARDWARE_VECTOR_SIZE: OnceLock<u64> = OnceLock::new();
    pub static HARDWARE_VECTOR_NAME: OnceLock<String> = OnceLock::new();

    pub fn log_vector_architecture() {
        let hardware_vector_size = vectors::get_hardware_vector_size();
        let hardware_vector_name = vectors::get_hardware_vector_name();

        Logger::get_instance().log(
            LogLevel::Info,
            format!(
                "CPU vector size for accelerated scans: {:?} bytes ({:?} bits), architecture: {}.",
                hardware_vector_size,
                hardware_vector_size * 8,
                hardware_vector_name,
            )
            .as_str(),
            None,
        );
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn detect_x86_features() -> (bool, u64, String) {
        if is_x86_feature_detected!("avx512f") {
            (true, 64, "avx512f".to_string())
        } else if is_x86_feature_detected!("avx2") {
            (true, 32, "avx2".to_string())
        } else if is_x86_feature_detected!("avx") {
            (true, 32, "avx".to_string())
        } else if is_x86_feature_detected!("sse4.2") {
            (true, 16, "sse4.2".to_string())
        } else if is_x86_feature_detected!("sse4.1") {
            (true, 16, "sse4.1".to_string())
        } else if is_x86_feature_detected!("ssse3") {
            (true, 16, "ssse3".to_string())
        } else if is_x86_feature_detected!("sse3") {
            (true, 16, "sse3".to_string())
        } else if is_x86_feature_detected!("sse2") {
            (true, 16, "sse2".to_string())
        } else if is_x86_feature_detected!("sse") {
            (true, 16, "sse".to_string())
        } else {
            (false, 0, "(none)".to_string())
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn detect_arm_features() -> (bool, u64, String) {
        // NEON is required for AArch64, so we can safely assume it's available
        // SVE and SVE2 detection would go here if needed
        if cfg!(target_feature = "neon") {
            (true, 16, "neon".to_string()) // NEON uses 128-bit vectors
        } else {
            (false, 0, "(none)".to_string())
        }
    }

    pub fn has_vector_support() -> bool {
        *HAS_VECTOR_SUPPORT.get_or_init(|| {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                detect_x86_features().0
            }
            #[cfg(target_arch = "aarch64")]
            {
                detect_arm_features().0
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
            {
                false
            }
        })
    }

    pub fn get_hardware_vector_size() -> u64 {
        *HARDWARE_VECTOR_SIZE.get_or_init(|| {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                detect_x86_features().1
            }
            #[cfg(target_arch = "aarch64")]
            {
                detect_arm_features().1
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
            {
                0
            }
        })
    }

    pub fn get_hardware_vector_name() -> &'static String {
        HARDWARE_VECTOR_NAME.get_or_init(|| {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                detect_x86_features().2
            }
            #[cfg(target_arch = "aarch64")]
            {
                detect_arm_features().2
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
            {
                "(none)".to_string()
            }
        })
    }
}
