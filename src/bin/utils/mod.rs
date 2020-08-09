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

#[allow(dead_code)]
pub fn parse_size(x: &str) -> result::Result<usize, String> {
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
    const MULTIPLIERS: [usize; 6] = [
        /* B */ 1,
        /* KiB */ 1024,
        /* MiB */ 1024 * 1024,
        /* GiB */ 1024 * 1024 * 1024,
        /* TiB */ 1024 * 1024 * 1024 * 1024,
        /* EiB */ 1024 * 1024 * 1024 * 1024 * 1024,
    ];

    let len = x.len();
    let unit = if len > 1 {
        let y = x.chars().rev().next().unwrap();
        if !y.is_digit(10) {
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

    let n = usize::from_str_radix(&x[..len - 1], 10).map_err(|e| e.to_string())?;
    if let Some(n) = n.checked_mul(MULTIPLIERS[unit as usize]) {
        Ok(n)
    } else {
        Err("Number to large to fit into usize".to_owned())
    }
}
