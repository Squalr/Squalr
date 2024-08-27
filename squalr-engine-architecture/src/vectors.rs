use std::arch::is_x86_feature_detected;
use std::sync::OnceLock;

pub mod vectors {
    use super::*;

    // Lazy static variable to cache the result of vector support check
    pub static HAS_VECTOR_SUPPORT: OnceLock<bool> = OnceLock::new();
    pub static HARDWARE_VECTOR_SIZE: OnceLock<u64> = OnceLock::new();

    pub fn has_vector_support() -> bool {
        *HAS_VECTOR_SUPPORT.get_or_init(|| {
            is_x86_feature_detected!("avx512f")
                || is_x86_feature_detected!("avx2")
                || is_x86_feature_detected!("avx")
                || is_x86_feature_detected!("sse4.2")
                || is_x86_feature_detected!("sse4.1")
                || is_x86_feature_detected!("ssse3")
                || is_x86_feature_detected!("sse3")
                || is_x86_feature_detected!("sse2")
                || is_x86_feature_detected!("sse")
        })
    }

    pub fn get_hardware_vector_size() -> u64 {
        *HARDWARE_VECTOR_SIZE.get_or_init(|| {
            if is_x86_feature_detected!("avx512f") {
                64 // AVX-512 uses 512-bit (64 bytes) vectors
            } else if is_x86_feature_detected!("avx2") {
                32 // AVX2 uses 256-bit (32 bytes) vectors
            } else if is_x86_feature_detected!("avx") {
                32 // AVX uses 256-bit (32 bytes) vectors
            } else if is_x86_feature_detected!("sse4.2") {
                16 // SSE4.2 uses 128-bit (16 bytes) vectors
            } else if is_x86_feature_detected!("sse4.1") {
                16 // SSE4.1 uses 128-bit (16 bytes) vectors
            } else if is_x86_feature_detected!("ssse3") {
                16 // SSSE3 uses 128-bit (16 bytes) vectors
            } else if is_x86_feature_detected!("sse3") {
                16 // SSE3 uses 128-bit (16 bytes) vectors
            } else if is_x86_feature_detected!("sse2") {
                16 // SSE2 uses 128-bit (16 bytes) vectors
            } else if is_x86_feature_detected!("sse") {
                16 // SSE uses 128-bit (16 bytes) vectors
            } else {
                0 // No SIMD support, or unhandled case
            }
        })
    }
}
