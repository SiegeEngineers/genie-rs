use genie::Campaign;
use std::path::PathBuf;
use std::fs::File;

fn main() {
    let filename = std::env::args().nth(1).expect("usage: extractcpx <filename>");
    let dir = std::env::args().nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir().expect("invalid cwd")
        });
    let dry_run = std::env::args().any(|arg| arg == "-l" || arg == "--list");

    let f = File::open(filename).expect("could not open file");
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

        if !dry_run {
            std::fs::write(dir.join(&names[i]), bytes).expect("failed to write");
        }
    }
}
