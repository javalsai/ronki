use std::path::PathBuf;

use clap::Parser;

/// MrKonqi made in rust (Ronki)
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(styles = get_clap_styles())]
pub struct Args {
    #[arg(short, default_value = "./config.toml")]
    pub config: PathBuf,

    /// Forcefully reset config
    #[arg(long, default_value_t = false)]
    pub reset_config: bool,
}

fn get_clap_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi256(anstyle::Ansi256Color(208)))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi256(anstyle::Ansi256Color(208)))),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Blue))),
        )
}
