// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, FixedOffset, Utc};
use serde::Deserialize;

/// Represents the decoded claims from a JWT payload.
///
/// # Overview
///
/// Extracts the standard and common claims from the middle segment
/// of a JWT (header.payload.signature). The payload is decoded from
/// base64url encoding and parsed as JSON.
///
/// # Examples
///
/// ```
/// use twc_rs::jwt::JwtPayload;
///
/// let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
/// let payload = JwtPayload::parse(token);
/// assert_eq!(payload.sub, Some("1234567890".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JwtPayload {
    /// Expiration time (Unix epoch seconds) — optional.
    pub exp:   Option<DateTime<FixedOffset>>,
    /// Issued-at time (Unix epoch seconds) — optional.
    pub iat:   Option<DateTime<FixedOffset>>,
    /// Scope string from the token — optional.
    pub scope: Option<String>,
    /// Subject (typically a user ID) — optional.
    pub sub:   Option<String>
}

/// Internal deserializer for JWT claims.
#[derive(Debug, Deserialize)]
struct RawClaims {
    exp:   Option<f64>,
    iat:   Option<f64>,
    scope: Option<String>,
    sub:   Option<String>
}

impl JwtPayload {
    /// Parse a JWT token string and extract its claims.
    ///
    /// # Overview
    ///
    /// Splits the token on `.` and base64url-decodes the second segment
    /// (the payload). The decoded JSON is then deserialized into the
    /// known claims. Unknown fields are silently ignored.
    ///
    /// # Arguments
    ///
    /// * `token` — A string slice containing the full JWT
    ///   (`header.payload.signature`).
    ///
    /// # Returns
    ///
    /// A [`JwtPayload`] with all recognized claims. Any field that is
    /// missing from the token or cannot be decoded is set to `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use twc_rs::jwt::JwtPayload;
    ///
    /// let payload = JwtPayload::parse("header.payload.signature");
    /// assert!(payload.sub.is_none());
    /// ```
    pub fn parse(token: &str) -> Self {
        let segments: Vec<&str> = token.splitn(3, '.').collect();
        if segments.len() < 2 {
            return empty();
        }

        let payload_b64 = segments[1];
        let Ok(decoded) = URL_SAFE_NO_PAD.decode(payload_b64) else {
            return empty();
        };

        let Ok(text) = String::from_utf8(decoded) else {
            return empty();
        };

        let Ok(raw) = serde_json::from_str::<RawClaims>(&text) else {
            return empty();
        };

        Self {
            exp:   raw.exp.and_then(timestamp_to_datetime),
            iat:   raw.iat.and_then(timestamp_to_datetime),
            scope: raw.scope,
            sub:   raw.sub
        }
    }

    /// Calculate the number of whole days remaining until the token expires.
    ///
    /// # Overview
    ///
    /// Returns `None` if the token has no expiration claim. Otherwise
    /// computes the difference between `exp` and the current system
    /// time, truncated to whole days.
    ///
    /// # Returns
    ///
    /// The number of days remaining (may be negative if already expired),
    /// or `None` when `exp` is absent.
    ///
    /// # Examples
    ///
    /// ```
    /// use twc_rs::jwt::JwtPayload;
    ///
    /// let payload = JwtPayload::parse("header.payload.signature");
    /// assert!(payload.days_remaining().is_none());
    /// ```
    #[must_use]
    pub fn days_remaining(&self) -> Option<i64> {
        let exp = self.exp?;
        let now = now_timestamp();
        let diff_secs = exp.timestamp() - now;
        Some(diff_secs / 86400)
    }

    /// Check whether the token expires within 30 days from now.
    ///
    /// # Overview
    ///
    /// Returns `true` if `exp` is set and falls within the next 30 days.
    /// Returns `false` if there is no expiration claim or the token
    /// expires further in the future.
    ///
    /// # Returns
    ///
    /// `true` if the token is expiring soon, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twc_rs::jwt::JwtPayload;
    ///
    /// let payload = JwtPayload::parse("header.payload.signature");
    /// assert!(!payload.is_expiring_soon());
    /// ```
    #[must_use]
    pub fn is_expiring_soon(&self) -> bool {
        let Some(exp) = self.exp else {
            return false;
        };
        let now = now_timestamp();
        let window_secs = 30 * 86400;
        exp.timestamp() > now && exp.timestamp() - now < window_secs
    }

    /// Check whether the token has already expired.
    ///
    /// # Overview
    ///
    /// Returns `true` if `exp` is set and is strictly before the current
    /// system time. Returns `false` if there is no expiration claim or
    /// the token is still valid.
    ///
    /// # Returns
    ///
    /// `true` if the token is expired, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twc_rs::jwt::JwtPayload;
    ///
    /// let payload = JwtPayload::parse("header.payload.signature");
    /// assert!(!payload.is_expired());
    /// ```
    #[must_use]
    pub fn is_expired(&self) -> bool {
        let Some(exp) = self.exp else {
            return false;
        };
        let now = now_timestamp();
        exp.timestamp() < now
    }
}

/// Return an empty [`JwtPayload`] with all fields set to `None`.
const fn empty() -> JwtPayload {
    JwtPayload {
        exp:   None,
        iat:   None,
        scope: None,
        sub:   None
    }
}

/// Convert a Unix timestamp (float seconds) to a timezone-aware datetime.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn timestamp_to_datetime(ts: f64) -> Option<DateTime<FixedOffset>> {
    let secs = ts as i64;
    let nanos = ((ts - secs as f64) * 1_000_000_000.0) as u32;
    let dt = DateTime::<Utc>::from_timestamp(secs, nanos)?;
    Some(dt.with_timezone(&FixedOffset::east_opt(0)?))
}

/// Return the current system time as Unix epoch seconds.
fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs().try_into().unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_token_with_claims() {
        use base64::Engine;

        let claims = serde_json::json!({
            "sub": "user-42",
            "scope": "read write",
            "iat": 1_700_000_000.0,
            "exp": 1_700_086_400.0
        });
        let payload_b64 = URL_SAFE_NO_PAD.encode(claims.to_string());
        let token = format!("header.{payload_b64}.signature");

        let payload = JwtPayload::parse(&token);
        assert_eq!(payload.sub, Some("user-42".to_string()));
        assert_eq!(payload.scope, Some("read write".to_string()));
        assert!(payload.exp.is_some());
        assert!(payload.iat.is_some());
    }

    #[test]
    fn parse_token_missing_claims() {
        let claims = serde_json::json!({});
        let payload_b64 = URL_SAFE_NO_PAD.encode(claims.to_string());
        let token = format!("header.{payload_b64}.sig");

        let payload = JwtPayload::parse(&token);
        assert_eq!(payload.sub, None);
        assert_eq!(payload.scope, None);
        assert!(payload.exp.is_none());
        assert!(payload.iat.is_none());
    }

    #[test]
    fn parse_invalid_base64_payload() {
        let token = "header.!!!invalid!!!.signature";
        let payload = JwtPayload::parse(token);
        assert!(payload.sub.is_none());
        assert!(payload.exp.is_none());
    }

    #[test]
    fn parse_too_few_segments() {
        let payload = JwtPayload::parse("headerpayload");
        assert!(payload.sub.is_none());
    }

    #[test]
    fn parse_invalid_utf8_payload() {
        let invalid_utf8 = vec![0xff, 0xfe, 0x00, 0x01];
        let payload_b64 = URL_SAFE_NO_PAD.encode(&invalid_utf8);
        let token = format!("header.{payload_b64}.sig");

        let result = JwtPayload::parse(&token);
        assert!(result.sub.is_none());
    }

    #[test]
    fn is_expired_false_when_no_exp() {
        let claims = serde_json::json!({"sub": "test"});
        let payload_b64 = URL_SAFE_NO_PAD.encode(claims.to_string());
        let token = format!("header.{payload_b64}.sig");

        let payload = JwtPayload::parse(&token);
        assert!(!payload.is_expired());
    }

    #[test]
    fn is_expiring_soon_false_when_no_exp() {
        let claims = serde_json::json!({"sub": "test"});
        let payload_b64 = URL_SAFE_NO_PAD.encode(claims.to_string());
        let token = format!("header.{payload_b64}.sig");

        let payload = JwtPayload::parse(&token);
        assert!(!payload.is_expiring_soon());
    }

    #[test]
    fn days_remaining_none_when_no_exp() {
        let claims = serde_json::json!({"sub": "test"});
        let payload_b64 = URL_SAFE_NO_PAD.encode(claims.to_string());
        let token = format!("header.{payload_b64}.sig");

        let payload = JwtPayload::parse(&token);
        assert!(payload.days_remaining().is_none());
    }

    #[test]
    fn is_expired_true_for_past_exp() {
        let claims = serde_json::json!({
            "exp": 1_000_000.0
        });
        let payload_b64 = URL_SAFE_NO_PAD.encode(claims.to_string());
        let token = format!("header.{payload_b64}.sig");

        let payload = JwtPayload::parse(&token);
        assert!(payload.is_expired());
    }

    #[test]
    #[expect(clippy::cast_precision_loss, clippy::suboptimal_flops)]
    fn is_expired_false_for_future_exp() {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let far_future = now_secs as f64 + 86400.0 * 365.0;
        let claims = serde_json::json!({"exp": far_future});
        let payload_b64 = URL_SAFE_NO_PAD.encode(claims.to_string());
        let token = format!("header.{payload_b64}.sig");

        let payload = JwtPayload::parse(&token);
        assert!(!payload.is_expired());
    }
}
