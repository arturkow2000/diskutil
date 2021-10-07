pub fn display_progress(left: u64, total: u64, bytes_per_second: f64) {
    let transferred = total - left;
    let bytes_per_second = bytes_per_second.round() as u64;
    let time_left = left / bytes_per_second;

    let minutes_left = time_left / 60;
    let seconds_left = time_left % 60;

    let percent = transferred as f32 * 100f32 / total as f32;

    eprintln!(
        "{:.2}% done ({} out of {}) transferred at {}/s ETA {:02}:{:02}",
        percent,
        transferred,
        total,
        crate::utils::size_to_string(bytes_per_second),
        minutes_left,
        seconds_left
    );
}
