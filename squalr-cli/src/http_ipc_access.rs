use anyhow::{Context, Result, bail};
use std::net::{SocketAddr, ToSocketAddrs};
use subtle::ConstantTimeEq;

/// One controller credential owns one worker session.
pub struct HttpIpcAccess {
    token: String,
}

impl HttpIpcAccess {
    pub fn from_environment() -> Result<Self> {
        let token = std::env::var("SQUALR_HTTP_IPC_TOKEN").context("Set SQUALR_HTTP_IPC_TOKEN to a random controller credential.")?;
        if token.len() < 32 || !token.bytes().all(|value| value.is_ascii_graphic()) {
            bail!("SQUALR_HTTP_IPC_TOKEN must contain at least 32 visible ASCII characters.");
        }
        Ok(Self { token })
    }

    pub fn loopback_address(value: &str) -> Result<SocketAddr> {
        let addresses = value.to_socket_addrs()?.collect::<Vec<_>>();
        if addresses.iter().any(|address| !address.ip().is_loopback()) {
            bail!("HTTP IPC requires a loopback address.");
        }
        addresses
            .first()
            .copied()
            .context("No loopback address resolved.")
    }

    pub fn authorizes(
        &self,
        header: Option<&str>,
    ) -> bool {
        header
            .and_then(|value| value.split_once(' '))
            .is_some_and(|(scheme, candidate)| scheme.eq_ignore_ascii_case("Bearer") && bool::from(candidate.as_bytes().ct_eq(self.token.as_bytes())))
    }
}

#[cfg(test)]
mod tests {
    use super::HttpIpcAccess;

    #[test]
    fn only_accepts_loopback_and_the_configured_bearer() {
        let access = HttpIpcAccess {
            token: "test-controller".into(),
        };
        assert!(HttpIpcAccess::loopback_address("[::1]:0").is_ok());
        assert!(HttpIpcAccess::loopback_address("0.0.0.0:0").is_err());
        assert!(access.authorizes(Some("Bearer test-controller")));
        assert!(!access.authorizes(None));
        assert!(!access.authorizes(Some("Bearer other")));
    }
}
