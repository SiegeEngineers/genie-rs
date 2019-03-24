use std::io::{stdout, Write};
use std::fs::{File, create_dir_all};
use std::path::PathBuf;
use genie_drs::DRSReader;
use quicli::prelude::*;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(name = "list")]
    /// List the resources in <file>
    List(List),
    #[structopt(name = "get")]
    /// Get a single resource by ID.
    Get(Get),
    #[structopt(name = "extract")]
    /// Extract the entire archive to a directory.
    Extract(Extract),
}

#[derive(StructOpt)]
struct List {
    #[structopt(parse(from_os_str))]
    /// Path to the .drs archive.
    archive: PathBuf,
}

#[derive(StructOpt)]
struct Get {
    #[structopt(parse(from_os_str))]
    /// Path to the .drs archive.
    archive: PathBuf,
    #[structopt(name = "resource")]
    /// The ID of the resource.
    resource_id: u32,
}

#[derive(StructOpt)]
struct Extract {
    #[structopt(parse(from_os_str))]
    /// Path to the .drs archive.
    archive: PathBuf,
    #[structopt(long, short = "t")]
    /// Only extract resources from this table.
    table: Option<String>,
    #[structopt(long, short = "o", parse(from_os_str))]
    /// Output directory to place the resources in.
    out: PathBuf,
}

fn list(args: List) -> CliResult {
    let mut file = File::open(args.archive)?;
    let drs = DRSReader::new(&mut file)?;

    for table in drs.tables() {
        for resource in table.resources() {
            println!("{}.{}", resource.id, table.resource_ext());
        }
    }

    Ok(())
}

fn get(args: Get) -> CliResult {
    let mut file = File::open(args.archive)?;
    let drs = DRSReader::new(&mut file)?;

    for table in drs.tables() {
        match table.get_resource(args.resource_id) {
            Some(ref resource) => {
                let buf = drs.read_resource(&mut file, table.resource_type, resource.id)?;
                stdout().write_all(&buf)?;
                return Ok(())
            },
            None => (),
        }
    }

    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Archive does not contain that resource").into())
}

fn extract(args: Extract) -> CliResult {
    let mut file = File::open(args.archive)?;
    let drs = DRSReader::new(&mut file)?;

    create_dir_all(&args.out)?;

    for table in drs.tables() {
        let table_ext = table.resource_ext();
        if let Some(ref filter_ext) = args.table {
            if &table_ext != filter_ext {
                continue;
            }
        }

        for resource in table.resources() {
            let buf = drs.read_resource(&mut file, table.resource_type, resource.id)?;
            let mut outfile = File::create(args.out.join(format!("{}.{}", resource.id, table_ext)))?;
            outfile.write_all(&buf)?;
        }
    }

    Ok(())
}

fn main() -> CliResult {
    let args = Cli::from_args();

    match args.command {
        Command::List(args) => list(args),
        Command::Get(args) => get(args),
        Command::Extract(args) => extract(args),
    }
}
