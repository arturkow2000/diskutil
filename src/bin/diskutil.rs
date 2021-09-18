use clap::{Clap, Subcommand};

mod cmd;
mod utils;

#[macro_use]
extern crate anyhow;

#[derive(Subcommand)]
enum Command {
    Create(cmd::create::Command),
    Gpt(cmd::gpt::Command),
}

#[derive(Clap)]
struct Options {
    #[clap(subcommand)]
    pub command: Command,
}

fn main() -> anyhow::Result<()> {
    let o = Options::parse();

    match o.command {
        Command::Create(c) => cmd::create::run(c),
        Command::Gpt(c) => cmd::gpt::run(c),
    }
}
