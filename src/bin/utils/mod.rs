use std::{io, result};

pub fn setup_logging(verbosity_level: u32) {
    use fern::colors::{Color, ColoredLevelConfig};

    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::BrightWhite)
        .trace(Color::Cyan);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            let color = colors.get_color(&record.level());
            let foreground = color.to_fg_str();
            let target = record.target();
            let level = record.level();

            let prefix = format!("[{}][{}]\x1b[{}m ", target, level, foreground);
            const SUFFIX: &str = "\x1b[0m";

            let s = format!("{}", message);
            let mut string_len = 0;
            let mut num_lines = 0;
            for x in s.split('\n') {
                string_len += x.len();
                num_lines += 1;
            }
            if num_lines == 0 {
                num_lines = 1;
            }

            let c = string_len + num_lines * (prefix.len() + SUFFIX.len()) + num_lines;
            let mut buf = String::with_capacity(c);
            for (i, line) in s.split('\n').enumerate() {
                buf += &prefix;
                buf += line;
                buf += SUFFIX;
                if i != num_lines - 1 {
                    buf.push('\n');
                }
            }

            debug_assert!(c >= buf.len());

            out.finish(format_args!("{}", buf))
        })
        .level(match verbosity_level {
            0 => log::LevelFilter::Error,
            1 => log::LevelFilter::Warn,
            2 => log::LevelFilter::Info,
            3 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
        .chain(io::stdout())
        .apply()
        .unwrap();
}

// TODO: floating point
#[allow(dead_code)]
pub fn parse_size(x: &str) -> result::Result<u64, String> {
    #[derive(Debug)]
    #[repr(u32)]
    enum Unit {
        B,
        KiB,
        MiB,
        GiB,
        TiB,
        EiB,
    };
    const MULTIPLIERS: [u64; 6] = [
        /* B */ 1,
        /* KiB */ 1024,
        /* MiB */ 1024 * 1024,
        /* GiB */ 1024 * 1024 * 1024,
        /* TiB */ 1024 * 1024 * 1024 * 1024,
        /* EiB */ 1024 * 1024 * 1024 * 1024 * 1024,
    ];

    let mut has_unit = false;
    let len = x.len();
    let unit = if len > 1 {
        let y = x.chars().rev().next().unwrap();
        if !y.is_digit(10) {
            has_unit = true;
            match y.to_ascii_uppercase() {
                'K' => Unit::KiB,
                'M' => Unit::MiB,
                'G' => Unit::GiB,
                'T' => Unit::TiB,
                'E' => Unit::EiB,
                _ => return Err("Unknown unit".to_owned()),
            }
        } else {
            Unit::B
        }
    } else {
        Unit::B
    };

    let n = if has_unit {
        u64::from_str_radix(&x[..len - 1], 10).map_err(|e| e.to_string())?
    } else {
        u64::from_str_radix(&x, 10).map_err(|e| e.to_string())?
    };
    if let Some(n) = n.checked_mul(MULTIPLIERS[unit as usize]) {
        Ok(n)
    } else {
        Err("Number to large to fit into usize".to_owned())
    }
}

// TODO: floating point
#[allow(dead_code)]
#[allow(non_upper_case_globals)]
pub fn size_to_string(s: u64) -> String {
    const KiB: u64 = 1024;
    const MiB: u64 = 1048576;
    const GiB: u64 = 1073741824;
    const TiB: u64 = 1099511627776;
    const EiB: u64 = 1125899906842624;

    match s {
        0..=1023 => format!("{}", s),
        1024..=1048575 => format!("{} KiB", s / 1024),
        1048576..=1073741823 => format!("{} MiB", s / 1048576),
        1073741824..=1099511627775 => format!("{} GiB", s / 1073741824),
        1099511627776..=1125899906842623 => format!("{} TiB", s / 1125899906842624),
        1125899906842624..=u64::MAX => format!("{} EiB", s / 1125899906842624),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_size;
    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("432").unwrap(), 432);
        assert_eq!(parse_size("432K").unwrap(), 432 * 1024);
        assert_eq!(parse_size("432M").unwrap(), 432 * 1024 * 1024);
        assert_eq!(parse_size("7G").unwrap(), 7 * 1024 * 1024 * 1024);
        assert_eq!(parse_size("0").unwrap(), 0);
        assert_eq!(parse_size("0E").unwrap(), 0);
    }
}
