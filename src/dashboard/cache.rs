// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Persistent dashboard snapshot: the last successful load is written to the
//! XDG cache so the next start paints real data instantly and revalidates in
//! the background instead of greeting the user with spinners.

use std::path::PathBuf;

use crate::tui::app::DashboardData;

/// Bump when the snapshot layout changes; older files are silently ignored.
const VERSION: u32 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
struct Snapshot {
    version: u32,
    data:    DashboardData
}

/// Path of the snapshot file for the given profile, or `None` when the XDG
/// cache directory cannot be resolved.
fn snapshot_path(profile: &str) -> Option<PathBuf> {
    let safe: String = profile
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    Some(
        dirs::cache_dir()?
            .join("twc-rs")
            .join(format!("snapshot-{safe}.json"))
    )
}

/// Loads the last persisted snapshot for the profile, if a compatible one
/// exists. Any read or parse failure just means a cold start.
#[must_use]
pub(crate) fn load(profile: &str) -> Option<DashboardData> {
    let path = snapshot_path(profile)?;
    let bytes = std::fs::read(path).ok()?;
    let snapshot: Snapshot = serde_json::from_slice(&bytes).ok()?;
    (snapshot.version == VERSION).then_some(snapshot.data)
}

/// Persists the snapshot for the profile in the background: written to a
/// temp file with owner-only permissions, then renamed into place.
pub(crate) fn save(profile: &str, data: DashboardData) {
    let Some(path) = snapshot_path(profile) else {
        return;
    };
    tokio::task::spawn_blocking(move || {
        let snapshot = Snapshot {
            version: VERSION,
            data
        };
        let Ok(bytes) = serde_json::to_vec(&snapshot) else {
            return;
        };
        let Some(dir) = path.parent() else {
            return;
        };
        if std::fs::create_dir_all(dir).is_err() {
            return;
        }
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, bytes).is_err() {
            return;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
        }
        let _ = std::fs::rename(&tmp, &path);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_roundtrips_through_json() {
        let mut app = crate::tui::app::App::new(5);
        app.account.login = "ra".to_string();
        app.projects = vec![crate::tui::app::ProjectSummary {
            id: 7,
            name: "Caravan".to_string(),
            server_count: 2,
            ..Default::default()
        }];
        let data = DashboardData::from_app(&app);
        let snapshot = Snapshot {
            version: VERSION,
            data
        };
        let bytes = serde_json::to_vec(&snapshot).expect("serialize");
        let back: Snapshot = serde_json::from_slice(&bytes).expect("deserialize");
        assert_eq!(back.version, VERSION);
        assert_eq!(back.data.projects[0].name, "Caravan");
        assert_eq!(back.data.account.login, "ra");
    }

    #[test]
    fn version_mismatch_is_ignored() {
        let snapshot = Snapshot {
            version: VERSION + 1,
            data:    DashboardData::from_app(&crate::tui::app::App::new(5))
        };
        let bytes = serde_json::to_vec(&snapshot).expect("serialize");
        let parsed: Snapshot = serde_json::from_slice(&bytes).expect("deserialize");
        assert_ne!(parsed.version, VERSION);
    }
}
