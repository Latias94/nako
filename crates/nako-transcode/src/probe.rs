use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct FfmpegProbeInventory {
    pub encoders: BTreeSet<String>,
    pub decoders: BTreeSet<String>,
    pub hwaccels: BTreeSet<String>,
    pub filters: BTreeSet<String>,
    pub bitstream_filters: BTreeSet<String>,
}

impl FfmpegProbeInventory {
    #[must_use]
    pub fn from_outputs(
        encoders: &str,
        decoders: &str,
        hwaccels: &str,
        filters: &str,
        bitstream_filters: &str,
    ) -> Self {
        Self {
            encoders: parse_ffmpeg_flagged_names(encoders),
            decoders: parse_ffmpeg_flagged_names(decoders),
            hwaccels: parse_ffmpeg_plain_names(hwaccels),
            filters: parse_ffmpeg_flagged_names(filters),
            bitstream_filters: parse_ffmpeg_plain_names(bitstream_filters),
        }
    }

    #[must_use]
    pub fn has_encoder(&self, name: &str) -> bool {
        self.encoders.contains(name)
    }

    #[must_use]
    pub fn has_decoder(&self, name: &str) -> bool {
        self.decoders.contains(name)
    }

    #[must_use]
    pub fn has_hwaccel(&self, name: &str) -> bool {
        self.hwaccels.contains(name)
    }

    #[must_use]
    pub fn has_filter(&self, name: &str) -> bool {
        self.filters.contains(name)
    }

    #[must_use]
    pub fn has_bitstream_filter(&self, name: &str) -> bool {
        self.bitstream_filters.contains(name)
    }
}

fn parse_ffmpeg_flagged_names(output: &str) -> BTreeSet<String> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.ends_with(':') || trimmed.starts_with('-') {
                return None;
            }

            let mut parts = trimmed.split_whitespace();
            let flags = parts.next()?;
            let name = parts.next()?;
            if name == "=" || !looks_like_ffmpeg_flag_column(flags) {
                return None;
            }
            Some(name.to_owned())
        })
        .collect()
}

fn parse_ffmpeg_plain_names(output: &str) -> BTreeSet<String> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.ends_with(':')
                || trimmed.starts_with('-')
                || trimmed.contains(' ')
            {
                return None;
            }
            Some(trimmed.to_owned())
        })
        .collect()
}

fn looks_like_ffmpeg_flag_column(value: &str) -> bool {
    value.len() >= 2
        && value
            .chars()
            .all(|ch| ch == '.' || ch.is_ascii_alphabetic())
        && value.chars().any(|ch| ch == '.')
}
