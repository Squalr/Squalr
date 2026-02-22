use ureq::{
    Agent,
    config::Config,
    tls::{TlsConfig, TlsProvider},
};

pub struct AppProvisionerHttpClient {}

impl AppProvisionerHttpClient {
    /// Creates a configured HTTP agent for app provisioning operations.
    pub fn create_agent() -> Agent {
        Config::builder()
            .tls_config(TlsConfig::builder().provider(Self::get_tls_provider()).build())
            .build()
            .new_agent()
    }

    #[cfg(target_os = "android")]
    fn get_tls_provider() -> TlsProvider {
        TlsProvider::Rustls
    }

    #[cfg(not(target_os = "android"))]
    fn get_tls_provider() -> TlsProvider {
        TlsProvider::NativeTls
    }
}
