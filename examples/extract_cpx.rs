extern crate genie;
extern crate structopt;

use genie::Campaign;
use std::{cmp, fs::File, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct List {
    /// Campaign file.
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug, StructOpt)]
struct Extract {
    /// Campaign file.
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    /// Output folder, defaults to cwd.
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "extractcpx", about = "Campaign file manager")]
enum Cli {
    /// List the scenario files in the campaign file.
    #[structopt(name = "list")]
    List(List),
    /// Extract scenario files from the campaign file.
    #[structopt(name = "extract")]
    Extract(Extract),
}

fn list(args: List) {
    let f = File::open(args.input).expect("could not open file");
    let mut campaign = Campaign::from(f).expect("not a campaign file");

    println!("Name: {}", campaign.name());
    println!("Version: {}", String::from_utf8_lossy(&campaign.version()));
    println!("Scenarios: ({})", campaign.len());

    let names = campaign
        .entries()
        .map(|entry| entry.filename.to_string())
        .collect::<Vec<String>>();

    (0..campaign.len()).for_each(|i| {
        let bytes = campaign.by_index_raw(i).expect("missing scenario data");
        println!("- {} ({})", names[i], format_bytes(bytes.len() as u32));
    });
}

fn extract(args: Extract) {
    let dir = args
        .output
        .unwrap_or_else(|| std::env::current_dir().expect("invalid cwd"));

    let f = File::open(args.input).expect("could not open file");
    let mut campaign = Campaign::from(f).expect("not a campaign file");

    let names = campaign
        .entries()
        .map(|entry| entry.filename.to_string())
        .collect::<Vec<String>>();

    (0..campaign.len()).for_each(|i| {
        let bytes = campaign.by_index_raw(i).expect("missing scenario data");
        println!("{}", names[i]);
        std::fs::write(dir.join(&names[i]), bytes).expect("failed to write");
    });
}

/// Derived from https://github.com/banyan/rust-pretty-bytes/blob/master/src/converter.rs
fn format_bytes(num: u32) -> String {
    let units = ["B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    if num < 1 {
        return format!("{} {}", num, "B");
    }
    let delimiter = 1000u32;
    let exponent = cmp::min(
        (f64::from(num).ln() / f64::from(delimiter).ln()).floor() as u32,
        (units.len() - 1) as u32,
    );
    let pretty_bytes = num / delimiter.pow(exponent);
    let unit = units[exponent as usize];
    format!("{:.2} {}", pretty_bytes, unit)
}

fn main() {
    match Cli::from_args() {
        Cli::List(args) => list(args),
        Cli::Extract(args) => extract(args),
    }
}
