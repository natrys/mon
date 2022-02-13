extern crate anyhow;
extern crate clap;
extern crate surge_ping;

use anyhow::Result;
use clap::{AppSettings, ArgEnum, Parser};
use std::time::Duration;

#[derive(ArgEnum, Clone, PartialEq)]
enum Notify {
    Up,
    Down,
}

#[derive(Parser)]
#[clap(version, about, long_about = None)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
#[clap(global_setting(AppSettings::DisableColoredHelp))]
struct Config {
    #[clap(
        short = 'i',
        long = "interval",
        default_value = "15",
        help = "Interval (in seconds) between heart beat pings",
        parse(try_from_str)
    )]
    interval: u64,

    #[clap(
        short = 'v',
        long = "verify",
        default_value = "5",
        help = "Verify status change for time (in seconds) before confirming",
        parse(try_from_str)
    )]
    verify: usize,

    #[clap(
        arg_enum,
        short = 'n',
        long = "notify",
        default_value = "down",
        help = "Notify when host goes up or down"
    )]
    notify: Notify,

    dest: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::parse();

    let mut pinger = surge_ping::pinger(config.dest.parse()?).await?;
    pinger.timeout(Duration::from_secs(1));

    let mut seq_cnt = 0;
    'outer: loop {
        let mut inner_count = 0;
        loop {
            inner_count += 1;
            if inner_count > config.verify {
                break 'outer;
            }

            seq_cnt += 1;
            if !(pinger.ping(seq_cnt).await.is_ok() ^ (config.notify == Notify::Down)) {
                break;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
        std::thread::sleep(Duration::from_secs(config.interval));
    }

    print!("\x07"); // ring terminal bell '\a'
    Ok(())
}
