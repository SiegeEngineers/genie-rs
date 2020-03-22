use genie::scx::{convert::AutoToWK, VersionBundle};
use genie::Scenario;
use std::{fs::File, path::PathBuf};
use structopt::StructOpt;

type CliResult = Result<(), Box<dyn std::error::Error>>;

/// Convert Age of Empires scenario files between versions.
#[derive(Debug, StructOpt)]
struct Cli {
    /// Input scenario file.
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    /// Output scenario file.
    #[structopt(parse(from_os_str))]
    output: PathBuf,
    /// Scenario version to output: 'aoe', 'ror', 'aoc', 'hd', 'wk'
    ///
    /// When setting the version to 'wk', HD edition and AoC scenarios will automatically be
    /// converted (swapping out unit types and terrains).
    version: Option<String>,
}

fn main() -> CliResult {
    let Cli {
        input,
        output,
        version,
    } = Cli::from_args();
    let version_arg = version;
    let version = match version_arg.as_ref().map(|s| &**s) {
        Some("aoe") => VersionBundle::aoe(),
        Some("ror") => VersionBundle::ror(),
        Some("aoc") => VersionBundle::aoc(),
        Some("hd") => VersionBundle::hd_edition(),
        Some("wk") => VersionBundle::userpatch_15(),
        Some(name) => panic!("unknown version {}", name),
        _ => VersionBundle::aoc(),
    };

    let instream = File::open(input)?;
    let mut scen = Scenario::read_from(instream)?;

    if version_arg == Some("wk".to_string()) {
        println!("Applying WololoKingdoms conversion...");
        let converter = AutoToWK::default();
        converter.convert(&mut scen)?;
    }

    let outstream = File::create(output)?;
    scen.write_to_version(outstream, &version)?;

    println!("Conversion complete!");

    Ok(())
}
