// Copyright (c) Victor Derks.
// SPDX-License-Identifier: MIT

use structopt::StructOpt;
use std::io;
use std::fs::File;
use std::io::BufReader;

mod jpeg_stream_reader;

#[derive(StructOpt)]
struct Cli {
    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() -> io::Result<()> {
    let args = Cli::from_args();

    println!("Dumping JPEG file: {:?}", args.path.to_str());
    println!("=============================================================================");

    let mut reader = BufReader::new(File::open(args.path)?);
    let mut jpeg_stream_reader = jpeg_stream_reader::JpegStreamReader::new(&mut reader);

    jpeg_stream_reader.dump()?;

    Ok(())
}
