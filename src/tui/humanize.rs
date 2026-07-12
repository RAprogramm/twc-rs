// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Human-readable formatting for API codes: locations become city names,
//! ISO timestamps become readable dates, raw megabytes become sensible units.

use std::borrow::Cow;

use rust_i18n::t;

/// Human name of a Timeweb location code (per the official docs), falling
/// back to the raw code for anything unknown. The code stays in parentheses
/// so support conversations still have the exact value.
#[must_use]
pub fn location(code: &str) -> String {
    let name: Cow<'_, str> = match code {
        "ru-1" => t!("location.ru_1"),
        "ru-2" => t!("location.ru_2"),
        "ru-3" => t!("location.ru_3"),
        "pl-1" => t!("location.pl_1"),
        "kz-1" => t!("location.kz_1"),
        "nl-1" => t!("location.nl_1"),
        "de-1" => t!("location.de_1"),
        _ => return code.to_string()
    };
    format!("{name} ({code})")
}

/// Readable date from an ISO 8601 timestamp: `2026-06-08T02:33:05.000Z`
/// becomes `2026-06-08 02:33`. Anything unparseable passes through.
#[must_use]
pub fn date(iso: &str) -> String {
    let Some((day, rest)) = iso.split_once('T') else {
        return iso.to_string();
    };
    let time: String = rest.chars().take(5).collect();
    format!("{day} {time}")
}

/// Megabytes as a sensible unit: whole gigabytes above 1 GB, else MB.
#[must_use]
pub fn megabytes(mb: i64) -> String {
    if mb >= 1024 && mb % 1024 == 0 {
        format!("{} GB", mb / 1024)
    } else if mb >= 1024 {
        #[expect(clippy::cast_precision_loss)]
        let gb = mb as f64 / 1024.0;
        format!("{gb:.1} GB")
    } else {
        format!("{mb} MB")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_location_gets_a_name_with_code() {
        let text = location("nl-1");
        assert!(text.contains("nl-1"));
        assert!(text.len() > "nl-1".len() + 3);
    }

    #[test]
    fn unknown_location_passes_through() {
        assert_eq!(location("xx-9"), "xx-9");
    }

    #[test]
    fn iso_date_becomes_readable() {
        assert_eq!(date("2026-06-08T02:33:05.000Z"), "2026-06-08 02:33");
        assert_eq!(date("garbage"), "garbage");
    }

    #[test]
    fn megabytes_scale_to_gigabytes() {
        assert_eq!(megabytes(512), "512 MB");
        assert_eq!(megabytes(20480), "20 GB");
        assert_eq!(megabytes(1536), "1.5 GB");
    }
}
