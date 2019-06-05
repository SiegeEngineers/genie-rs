use std::{
    io::{self, stdout, Write},
    fs::{File, create_dir_all},
    path::PathBuf,
    collections::HashSet,
};
use genie_drs::{DRSReader, DRSWriter, WriteStrategy};
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
    #[structopt(name = "add")]
    /// Add a resource to an existing archive.
    Add(Add),
}

#[derive(StructOpt)]
struct List {
    #[structopt(parse(from_os_str))]
    /// Path to the .drs archive.
    archive: PathBuf,
}

#[derive(StructOpt)]
struct Get {
    /// Path to the .drs archive.
    #[structopt(parse(from_os_str))]
    archive: PathBuf,
    /// The ID of the resource.
    #[structopt(name = "resource")]
    resource_id: u32,
}

#[derive(StructOpt)]
struct Extract {
    /// Path to the .drs archive.
    #[structopt(parse(from_os_str))]
    archive: PathBuf,
    /// Only extract resources from this table.
    #[structopt(long, short = "t")]
    table: Option<String>,
    /// Output directory to place the resources in.
    #[structopt(long, short = "o", parse(from_os_str))]
    out: PathBuf,
}

#[derive(Debug, StructOpt)]
struct Add {
    /// Path to the .drs archive.
    #[structopt(parse(from_os_str))]
    archive: PathBuf,
    /// Table to add the file to.
    #[structopt(long, short = "t", number_of_values = 1)]
    table: Vec<String>,
    /// ID of the file.
    #[structopt(long, short = "i", number_of_values = 1)]
    id: Vec<u32>,
    /// Path to the file to add. `-` for standard input.
    #[structopt(parse(from_os_str), default_value = "-")]
    file: Vec<PathBuf>,
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

    Err(io::Error::new(io::ErrorKind::NotFound, "Archive does not contain that resource").into())
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

fn add(args: Add) -> CliResult {
    assert_eq!(args.file.len(), args.table.len());
    assert_eq!(args.file.len(), args.id.len());
    let mut output_file = args.archive.clone();
    output_file.set_file_name(format!("{}.{}", args.archive.file_name().unwrap().to_str().unwrap(), "temp"));
    let mut input = File::open(args.archive)?;
    let output = File::create(output_file)?;
    let drs_read = DRSReader::new(&mut input)?;
    let (tables, files) = drs_read.tables().fold((0, 0), |(tables, files), table| (tables + 1, files + table.len() as u32));

    let new_tables = args.table.iter().fold(HashSet::new(), |mut uniq, table| {
        uniq.insert(table);
        uniq
    }).len() as u32;
    let new_files = args.id.len() as u32;

    let mut drs_write = DRSWriter::new(output,
        WriteStrategy::ReserveDirectory(tables + new_tables, files + new_files))?;

    for t in drs_read.tables() {
        for r in t.resources() {
            let b = drs_read.read_resource(&mut input, t.resource_type, r.id)?;
            drs_write.add(t.resource_type, r.id, b.as_ref())?;
        }
    }

    for (i, path) in args.file.iter().enumerate() {
        let mut res_type = [0x20; 4];
        let slice = args.table[i].as_bytes();
        (&mut res_type[0..slice.len()]).copy_from_slice(slice);
        res_type.reverse();
        drs_write.add(res_type, args.id[i], File::open(path)?)?;
    }

    drs_write.flush()?;

    Ok(())
}

fn main() -> CliResult {
    let args = Cli::from_args();

    match args.command {
        Command::List(args) => list(args),
        Command::Get(args) => get(args),
        Command::Extract(args) => extract(args),
        Command::Add(args) => add(args),
    }
}
