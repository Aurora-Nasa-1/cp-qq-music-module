pub(crate) mod netease;
mod tencent;

pub(crate) use netease::NeteaseClient;
pub(crate) use tencent::{TencentClient, TencentMqttLoginSession};

fn collect_into<T, U>(values: Vec<T>) -> Vec<U>
where
    T: Into<U>,
{
    values.into_iter().map(Into::into).collect()
}

fn parse_lrc_timestamp_tag(segment: &str) -> Option<(usize, String)> {
    if !segment.starts_with('[') {
        return None;
    }
    let end = segment.find(']')?;
    let inner = &segment[1..end];
    let (minute, sec_part) = inner.split_once(':')?;
    if minute.len() != 2 || !minute.as_bytes().iter().all(u8::is_ascii_digit) {
        return None;
    }

    let (second, ms_part) = match sec_part.split_once('.') {
        Some((second, ms_part)) => (second, ms_part),
        None => (sec_part, ""),
    };
    if second.len() != 2 || !second.as_bytes().iter().all(u8::is_ascii_digit) {
        return None;
    }
    if !ms_part.is_empty() && !ms_part.as_bytes().iter().all(u8::is_ascii_digit) {
        return None;
    }

    let millisecond = match ms_part.len() {
        0 => "000".to_string(),
        1 => format!("{ms_part}00"),
        2 => format!("{ms_part}0"),
        _ => ms_part[..3].to_string(),
    };

    Some((end + 1, format!("[{minute}:{second}.{millisecond}]")))
}

fn normalize_lrc_line(line: &str) -> Option<String> {
    let line = line.trim_start_matches('\u{feff}');
    let mut cursor = 0;
    let mut normalized = String::new();
    while let Some((consumed, tag)) = parse_lrc_timestamp_tag(&line[cursor..]) {
        normalized.push_str(tag.as_str());
        cursor += consumed;
    }
    if normalized.is_empty() {
        return None;
    }
    normalized.push_str(&line[cursor..]);
    Some(normalized)
}

fn normalize_timestamp_lyric(raw: String) -> String {
    raw.lines().filter_map(normalize_lrc_line).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::normalize_timestamp_lyric;

    #[test]
    fn normalize_timestamp_lyric_should_keep_only_timed_lines() {
        let raw = "[ti:sample]\n[ar:test]\n{\"t\":0,\"c\":[{\"tx\":\"作词\"}]}\n[00:00.27]line \
                   a\n[00:10.3]line b\n[00:11]line c\n[00:20.12][00:30.456]line d";

        let normalized = normalize_timestamp_lyric(raw.to_string());
        assert_eq!(
            normalized,
            "[00:00.270]line a\n[00:10.300]line b\n[00:11.000]line c\n[00:20.120][00:30.456]line d"
        );
    }
}
