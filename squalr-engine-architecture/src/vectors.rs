pub mod vectors {
    use std::arch::is_x86_feature_detected;

    pub fn has_vector_support() -> bool {
        return is_x86_feature_detected!("avx512f")
            || is_x86_feature_detected!("avx2")
            || is_x86_feature_detected!("avx")
            || is_x86_feature_detected!("sse4.2")
            || is_x86_feature_detected!("sse4.1")
            || is_x86_feature_detected!("ssse3")
            || is_x86_feature_detected!("sse3")
            || is_x86_feature_detected!("sse2")
            || is_x86_feature_detected!("sse");
    }

    pub fn get_hardware_vector_size() -> usize {
        if is_x86_feature_detected!("avx512f") {
            return 64; // AVX-512 uses 512-bit (64 bytes) vectors
        } else if is_x86_feature_detected!("avx2") {
            return 32; // AVX2 uses 256-bit (32 bytes) vectors
        } else if is_x86_feature_detected!("avx") {
            return 32; // AVX uses 256-bit (32 bytes) vectors
        } else if is_x86_feature_detected!("sse4.2") {
            return 16; // SSE4.2 uses 128-bit (16 bytes) vectors
        } else if is_x86_feature_detected!("sse4.1") {
            return 16; // SSE4.1 uses 128-bit (16 bytes) vectors
        } else if is_x86_feature_detected!("ssse3") {
            return 16; // SSSE3 uses 128-bit (16 bytes) vectors
        } else if is_x86_feature_detected!("sse3") {
            return 16; // SSE3 uses 128-bit (16 bytes) vectors
        } else if is_x86_feature_detected!("sse2") {
            return 16; // SSE2 uses 128-bit (16 bytes) vectors
        } else if is_x86_feature_detected!("sse") {
            return 16; // SSE uses 128-bit (16 bytes) vectors
        } else {
            return 0; // No SIMD support, or unhandled case
        }
    }
}
