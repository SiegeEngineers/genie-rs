#![allow(unused_imports)]
#![allow(dead_code)]

extern crate nom;
use anyhow::Context;
use flate2::bufread::DeflateDecoder;
use genie_rec::actions::Action;
use genie_rec::RecordedGame;
use nom::{
    bytes::complete::{tag, take_while_m_n},
    combinator::map_res,
    error::dbg_dmp,
    error::Error,
    error::FromExternalError,
    sequence::tuple,
    IResult,
};
use nom_derive::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufRead;
use std::io::Read;
use std::io::{stdout, Seek, SeekFrom};
use std::{env::args, io::BufReader};

fn main() {
    dump().context("Failed to dump recording").unwrap();
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct RecordingFile(Vec<u8>);

#[track_caller]
fn dump() -> Result<(), anyhow::Error> {
    let file = include_bytes!("SD-AgeIIDE_Replay_181966005.aoe2record");
    let decoded: RecordingFile = bincode::deserialize(&file[..]).unwrap();

    let mut deflate_buf_reader = DeflateDecoder::new(BufReader::new(decoded.0.as_slice()));

    // for byte in deflate_buf_reader {
    //     println!("{byte:?}");
    // }
    let mut header = vec![];
    deflate_buf_reader.read_to_end(&mut header)?;
    std::fs::write(r"header2.bin", header)?;

    Ok(())
}
