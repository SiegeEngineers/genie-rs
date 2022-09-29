#![allow(unused_imports)]
#![allow(dead_code)]

use anyhow::Context;
use genie_rec::actions::Action;
use genie_rec::RecordedGame;
use std::env::args;
use std::fs::File;
use std::io::{Seek, SeekFrom};

fn main() {
    dump().context("Failed to dump recording").unwrap();
}

#[track_caller]
fn dump() -> Result<(), anyhow::Error> {
    let mut args = args();
    // skip executable
    dbg!(&args);
    args.next();
    let filename = args
        .next()
        .expect("Please give a filename of a record to dump");

    let mut f = File::open(filename)?;
    let mut r = RecordedGame::new(&mut f)?;
    println!("version, {}", r.save_version());
    let header = r.get_header_data()?;
    std::fs::write(r"header.bin", header)?;
    match r.header() {
        Ok(_) => {}
        Err(err) => {
            println!("Failed parsing header: {}", err);
        }
    }
    // for act in r.actions()? {
    //     match act {
    //         Ok(Action::Command(command)) => {
    //             println!("{:#?}", command);
    //         }
    //         Ok(Action::Chat(chat)) => {
    //             println!("{:#?}", chat);
    //         }
    //         Ok(Action::Embedded(embedded)) => {
    //             println!("{:#?}", embedded);
    //         }
    //
    //         Ok(_) => {}
    //         Err(err) => {
    //             println!("Position: {:?}", f.seek(SeekFrom::Current(0)));
    //             return Err(err.into());
    //         }
    //     }
    // }
    Ok(())
}
