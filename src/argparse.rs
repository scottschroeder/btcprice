use clap::Parser;

pub fn get_args() -> CliOpts {
    CliOpts::parse()
}

#[derive(Parser, Debug)]
#[clap(version = clap::crate_version!(), author = "Scott S. <scottschroeder@sent.com>")]
pub struct CliOpts {
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    Btc(Btc),
    #[clap(name = "mbtc")]
    MBtc(Mbtc),
    Test(Test),
}

#[derive(Parser, Debug)]
pub struct Btc {}
#[derive(Parser, Debug)]
pub struct Mbtc {}
#[derive(Parser, Debug)]
pub struct Test {}
