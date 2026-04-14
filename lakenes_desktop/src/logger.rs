use anstyle::{AnsiColor, Style};
use chrono::Local;
use env_logger::fmt::Formatter;
use log::{Level, Record};
use std::io::Write;

fn format_log(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    let time = Local::now().format("%H:%M:%S%.3f");
    let time_style = Style::new().fg_color(Some(AnsiColor::BrightBlack.into()));

    let level_style = match record.level() {
        Level::Trace => Style::new().fg_color(Some(AnsiColor::Magenta.into())),
        Level::Debug => Style::new().fg_color(Some(AnsiColor::Blue.into())),
        Level::Info => Style::new().fg_color(Some(AnsiColor::Green.into())),
        Level::Warn => Style::new().fg_color(Some(AnsiColor::Yellow.into())).bold(),
        Level::Error => Style::new().fg_color(Some(AnsiColor::Red.into())).bold(),
    };

    let module = record
        .module_path()
        .unwrap_or("sys")
        .rsplit("::")
        .next()
        .unwrap_or("sys")
        .to_uppercase();

    let module_style = Style::new().fg_color(Some(AnsiColor::Cyan.into())).bold();

    writeln!(
        buf,
        "{}{}{} {}{}{} ({}{}{}) {}",
        time_style.render(),
        time,
        time_style.render_reset(),
        level_style.render(),
        record.level(),
        level_style.render_reset(),
        module_style.render(),
        module,
        module_style.render_reset(),
        record.args()
    )
}

pub fn init_logger(verbose: bool) {
    let mut builder = env_logger::Builder::new();

    builder.format(format_log);

    if verbose {
        builder.filter_level(log::LevelFilter::Trace);
    } else {
        builder.filter_level(log::LevelFilter::Info);
    }

    builder.init();
}
