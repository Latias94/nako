use nako_core::{TranscodeSessionRuntimeMetrics, TranscodeSessionRuntimeProgress};

#[must_use]
pub fn parse_ffmpeg_progress_report(output: &[u8]) -> TranscodeSessionRuntimeMetrics {
    let text = String::from_utf8_lossy(output);
    let mut current = TranscodeSessionRuntimeMetrics::default();
    let mut latest = TranscodeSessionRuntimeMetrics::default();

    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        apply_progress_field(&mut current, key.trim(), value.trim());
        if key.trim() == "progress" {
            latest = current.clone();
            current = TranscodeSessionRuntimeMetrics::default();
        }
    }

    if latest.is_empty() { current } else { latest }
}

fn apply_progress_field(metrics: &mut TranscodeSessionRuntimeMetrics, key: &str, value: &str) {
    match key {
        "frame" => metrics.frame_count = parse_u64(value),
        "fps" => metrics.fps_millis = parse_decimal_millis(value),
        "bitrate" => metrics.bitrate_kbps = parse_bitrate_kbps(value),
        "total_size" => metrics.total_size_bytes = parse_u64(value),
        "out_time_us" | "out_time_ms" => {
            metrics.output_time_ms = parse_u64(value).map(|value| value / 1_000);
        }
        "out_time" => metrics.output_time_ms = parse_out_time_ms(value),
        "dup_frames" => metrics.dup_frames = parse_u64(value),
        "drop_frames" => metrics.drop_frames = parse_u64(value),
        "speed" => metrics.speed_millis = parse_speed_millis(value),
        "progress" => {
            metrics.progress = match value {
                "continue" => Some(TranscodeSessionRuntimeProgress::Continue),
                "end" => Some(TranscodeSessionRuntimeProgress::End),
                _ => None,
            };
        }
        _ => {}
    }
}

fn parse_u64(value: &str) -> Option<u64> {
    value.trim().parse().ok()
}

fn parse_decimal_millis(value: &str) -> Option<u64> {
    let value = value.trim();
    if value == "N/A" {
        return None;
    }

    value
        .parse::<f64>()
        .ok()
        .map(|value| (value * 1_000.0).round() as u64)
}

fn parse_bitrate_kbps(value: &str) -> Option<u64> {
    let value = value.trim().trim_end_matches("kbits/s").trim();
    parse_decimal_millis(value).map(|value| value / 1_000)
}

fn parse_speed_millis(value: &str) -> Option<u64> {
    parse_decimal_millis(value.trim().trim_end_matches('x'))
}

fn parse_out_time_ms(value: &str) -> Option<u64> {
    let mut parts = value.trim().split(':');
    let hours = parts.next()?.parse::<u64>().ok()?;
    let minutes = parts.next()?.parse::<u64>().ok()?;
    let seconds = parts.next()?;
    if parts.next().is_some() {
        return None;
    }

    let (seconds, fraction) = seconds.split_once('.').unwrap_or((seconds, "0"));
    let seconds = seconds.parse::<u64>().ok()?;
    let mut millis = fraction.chars().take(3).collect::<String>();
    while millis.len() < 3 {
        millis.push('0');
    }
    let millis = millis.parse::<u64>().unwrap_or(0);

    Some((((hours * 60) + minutes) * 60 + seconds) * 1_000 + millis)
}
