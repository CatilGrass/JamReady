use env_logger::Builder;
use env_logger::fmt::Color;
use env_logger::fmt::Color::{Cyan, Green, Red, White, Yellow};
use log::{Level, LevelFilter};

/// 构建 Logger
pub fn logger_build(level: LevelFilter, short: bool) {
    if short {
        build_short(level);
    } else {
        build_full(level)
    }
}

fn build_full(level: LevelFilter) {
    Builder::new()
        .format(|buf, record| {
            use std::io::Write;

            let now = chrono::Local::now();

            let mut gray = buf.style();
            gray.set_color(Color::Rgb(105, 105, 105)).set_bold(true);

            let mut level = buf.style();
            level.set_color(
                match record.level() {
                    Level::Error => { Red }
                    Level::Warn => { Yellow }
                    Level::Info => { Green }
                    Level::Debug => { Green }
                    Level::Trace => { Cyan }
                }
            ).set_bold(
                match record.level() {
                    Level::Error => { true }
                    Level::Warn => { false }
                    Level::Info => { false }
                    Level::Debug => { false }
                    Level::Trace => { false }
                }
            );

            let mut output = buf.style();
            output.set_color(
                match record.level() {
                    Level::Error => { Red }
                    Level::Warn => { Yellow }
                    Level::Info => { White }
                    Level::Debug => { White }
                    Level::Trace => { Cyan }
                }
            ).set_bold(
                match record.level() {
                    Level::Error => { true }
                    Level::Warn => { true }
                    Level::Info => { false }
                    Level::Debug => { false }
                    Level::Trace => { false }
                }
            );

            writeln!(
                buf,
                "{} {} {} : {}",
                gray.value(now.format("%H:%M:%S")),
                gray.value(record.module_path().unwrap_or("")),
                level.value(record.level()),
                output.value(record.args())
            )

        })
        .filter(None, level)
        .init();
}

fn build_short(level: LevelFilter) {
    Builder::new()
        .format(|buf, record| {
            use std::io::Write;

            let mut level = buf.style();
            level.set_color(
                match record.level() {
                    Level::Error => { Red }
                    Level::Warn => { Yellow }
                    Level::Info => { Green }
                    Level::Debug => { Green }
                    Level::Trace => { Cyan }
                }
            ).set_bold(
                match record.level() {
                    Level::Error => { true }
                    Level::Warn => { false }
                    Level::Info => { false }
                    Level::Debug => { false }
                    Level::Trace => { false }
                }
            );

            let mut output = buf.style();
            output.set_color(
                match record.level() {
                    Level::Error => { Red }
                    Level::Warn => { Yellow }
                    Level::Info => { White }
                    Level::Debug => { White }
                    Level::Trace => { Cyan }
                }
            ).set_bold(
                match record.level() {
                    Level::Error => { true }
                    Level::Warn => { true }
                    Level::Info => { false }
                    Level::Debug => { false }
                    Level::Trace => { false }
                }
            );

            writeln!(
                buf,
                "{} {}",
                level.value(record.level()),
                output.value(record.args())
            )

        })
        .filter(None, level)
        .init();
}