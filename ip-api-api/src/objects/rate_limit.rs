//! https://ip-api.com/docs/api:json

pub const RESPONSE_HEADER_KEY_X_RL: &str = "X-Rl";
pub const RESPONSE_HEADER_KEY_X_TTL: &str = "X-Ttl";

#[derive(Debug, Copy, Clone)]
pub struct RateLimit {
    pub remaining: Option<usize>,
    pub seconds_until_reset: Option<usize>,
}
