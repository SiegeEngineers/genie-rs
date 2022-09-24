use anyhow::Context;
use genie_rec::actions::Action;
use genie_rec::RecordedGame;
use std::env::args;
use std::fs::File;
use std::io::{stdout, Seek, SeekFrom};

fn main() {
    dump().context("Failed to dump recording").unwrap();
}

#[track_caller]
fn dump() -> Result<(), anyhow::Error> {
    let mut args = args();
    // skip executable
    args.next();
    let filename = args
        .next()
        .expect("Please give a filename of a record to dump");

    let mut f = File::open(filename)?;
    let mut r = RecordedGame::new(&mut f)?;
    println!("{:?}", r.header()?);
    let mut header = r.get_header_deflate()?;
    std::io::copy(&mut header, &mut stdout())?;
    Ok(())
}
