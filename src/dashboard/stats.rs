// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Live statistics fetching for the dashboard sparklines.

use timeweb_rs::authenticated;

use crate::tui;

/// Spawns a background task that fetches live statistics for a resource and
/// sends them back to the event loop.
pub fn spawn_stats_fetch(
    tx: tokio::sync::mpsc::UnboundedSender<tui::event::AppEvent>,
    token: String,
    req: tui::app::StatsRequest
) {
    tokio::spawn(async move {
        let config = authenticated(token);
        match fetch_resource_stats(&config, &req).await {
            Ok(stats) => {
                if stats.cpu.is_empty()
                    && stats.ram.is_empty()
                    && stats.net_in.is_empty()
                    && stats.net_out.is_empty()
                {
                    let _ = tx.send(tui::event::AppEvent::StatsError(format!(
                        "stats {}: no data points returned",
                        req.name
                    )));
                }
                let _ = tx.send(tui::event::AppEvent::Stats(Box::new(stats)));
            }
            Err(e) => {
                let _ = tx.send(tui::event::AppEvent::StatsError(format!(
                    "stats {}: {e}",
                    req.name
                )));
            }
        }
    });
}

/// Keeps the most recent `LIVE_STATS_SAMPLES` points of a series.
const fn tail_skip(len: usize) -> usize {
    const LIVE_STATS_SAMPLES: usize = 60;
    len.saturating_sub(LIVE_STATS_SAMPLES)
}

/// Fetches live statistics for the selected server (current, non-deprecated
/// endpoint: CPU and network) or app (CPU, RAM and network) and maps them to a
/// unified [`tui::app::ResourceStats`] series.
async fn fetch_resource_stats(
    config: &timeweb_rs::apis::configuration::Configuration,
    req: &tui::app::StatsRequest
) -> Result<tui::app::ResourceStats, String> {
    use tui::app::ResourceTab;

    match req.tab {
        ResourceTab::Servers => fetch_server_stats(config, req).await,
        ResourceTab::Apps => fetch_app_stats(config, req).await,
        _ => Err("live statistics are not available for this resource".to_string())
    }
}

/// Formats a UTC timestamp the way the statistics endpoints expect: ISO 8601
/// without a timezone offset (`2023-05-25T14:35:38`).
fn stats_timestamp(at: chrono::DateTime<chrono::Utc>) -> String {
    at.format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// Fetches server statistics via the current endpoint, which reports CPU load
/// and network throughput (it does not expose live RAM usage).
async fn fetch_server_stats(
    config: &timeweb_rs::apis::configuration::Configuration,
    req: &tui::app::StatsRequest
) -> Result<tui::app::ResourceStats, String> {
    let id: i32 = req
        .id
        .parse()
        .map_err(|_| format!("invalid server id {}", req.id))?;
    let now = chrono::Utc::now();
    let time_from = stats_timestamp(now - chrono::Duration::hours(24));
    let keys = "system.cpu.util;network.request;network.response";

    let resp = timeweb_rs::apis::servers_api::get_server_statistics_new(
        config, id, &time_from, "24", keys
    )
    .await
    .map_err(|e| e.to_string())?;

    let mut stats = tui::app::ResourceStats {
        id: req.id.clone(),
        subject: req.name.clone(),
        ..Default::default()
    };

    for series in resp.statistics.into_iter().flatten() {
        let Some(name) = series.name.as_deref() else {
            continue;
        };
        let mut values: Vec<f64> = series
            .list
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.value)
            .collect();
        let values = values.split_off(tail_skip(values.len()));
        match name {
            "system.cpu.util" => stats.cpu = values,
            "network.request" => stats.net_in = values,
            "network.response" => stats.net_out = values,
            _ => {}
        }
    }

    Ok(stats)
}

/// Fetches app statistics, which report CPU load, RAM usage and network
/// throughput as a single time-series response.
async fn fetch_app_stats(
    config: &timeweb_rs::apis::configuration::Configuration,
    req: &tui::app::StatsRequest
) -> Result<tui::app::ResourceStats, String> {
    let now = chrono::Utc::now();
    let from = stats_timestamp(now - chrono::Duration::hours(24));
    let to = stats_timestamp(now);

    let resp = timeweb_rs::apis::apps_api::get_app_statistics(config, &req.id, &from, &to)
        .await
        .map_err(|e| e.to_string())?;

    let cpu: Vec<f64> = resp
        .cpu
        .iter()
        .skip(tail_skip(resp.cpu.len()))
        .map(|c| c.load)
        .collect();
    let ram: Vec<f64> = resp
        .ram
        .iter()
        .skip(tail_skip(resp.ram.len()))
        .map(|r| {
            if r.total > 0.0 {
                (r.used / r.total) * 100.0
            } else {
                0.0
            }
        })
        .collect();
    let net = &resp.network_traffic;
    let net_in: Vec<f64> = net
        .iter()
        .skip(tail_skip(net.len()))
        .map(|n| n.incoming)
        .collect();
    let net_out: Vec<f64> = net
        .iter()
        .skip(tail_skip(net.len()))
        .map(|n| n.outgoing)
        .collect();

    Ok(tui::app::ResourceStats {
        id: req.id.clone(),
        subject: req.name.clone(),
        cpu,
        ram,
        net_in,
        net_out
    })
}
