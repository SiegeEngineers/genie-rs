use genie::Campaign;
use structopt::StructOpt;
use std::path::PathBuf;
use std::fs::File;

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

    let names = campaign.entries()
        .map(|entry| entry.filename.to_string())
        .collect::<Vec<String>>();

    for i in 0..campaign.len() {
        let bytes = campaign.by_index_raw(i).expect("missing scenario data");
        println!("- {} ({} B)", names[i], bytes.len());
    }
}

fn extract(args: Extract) {
    let dir = args.output.unwrap_or_else(|| {
        std::env::current_dir().expect("invalid cwd")
    });

    let f = File::open(args.input).expect("could not open file");
    let mut campaign = Campaign::from(f).expect("not a campaign file");

    let names = campaign.entries()
        .map(|entry| entry.filename.to_string())
        .collect::<Vec<String>>();

    for i in 0..campaign.len() {
        let bytes = campaign.by_index_raw(i).expect("missing scenario data");
        println!("{}", names[i]);
        std::fs::write(dir.join(&names[i]), bytes).expect("failed to write");
    }
}

fn main() {
    match Cli::from_args() {
        Cli::List(args) => list(args),
        Cli::Extract(args) => extract(args),
    }
}
