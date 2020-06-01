use genie::RecordedGame;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

/// Print out all the actions stored in a recorded game file body.
#[derive(StructOpt)]
struct Cli {
    /// Path to the recorded game file.
    filename: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let Cli { filename } = Cli::from_args();

    let file = File::open(filename)?;
    let mut rec = RecordedGame::new(file)?;
    for action in rec.actions()? {
        println!("{:?}", action?);
    }

    Ok(())
}
