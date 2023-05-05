use structopt::StructOpt;
use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

//#[allow(dead_code)]

pub struct JpegStreamReader<'a> {
    buf_reader: & 'a mut ( dyn Read + 'a),
    jpegls_stream: bool
}

impl<'a> JpegStreamReader<'a> {
    pub fn new(the_buf_reader: &'a mut dyn Read) -> JpegStreamReader<'a> {
        Self { buf_reader: the_buf_reader, jpegls_stream: false }
    }

    pub fn dump(&mut self) -> Result<(), io::Error> {
        let mut value= self.read_byte()?;
        while value != -1 {
            if value == 0xFF {
                let marker_code = self.read_byte()?;
                if self.is_marker_code(marker_code) {
                    self.dump_marker(marker_code);
                }
            }
            value= self.read_byte()?
        }

        Ok(())
    }

    fn is_marker_code(&self, code: i32) -> bool
    {
        // To prevent marker codes in the encoded bit stream encoders must encode the next byte zero or the next bit zero (jpeg-ls).
        if self.jpegls_stream {
            return (code & 0x80) == 0x80;
        }

        code > 0
    }

    fn dump_marker(&self, marker_code: i32) {
        match marker_code {
            _ => println!("{:>8} Marker 0xFF{:X}", JpegStreamReader::get_start_offset(), marker_code)
        }
    }

    fn get_start_offset() -> i32 {
        return 0; // TODO
    }

    fn read_byte(&mut self) -> Result<i32, io::Error> {
        let mut buffer = [0; 1];

        let n = self.buf_reader.read(&mut buffer[..])?;
        if n == 0 {
            return Ok(-1);
        }

        Ok(buffer[0] as i32)
    }


}

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

    let f = File::open(args.path)?;
    let mut reader = BufReader::new(f);

    let mut jpeg_stream_reader = JpegStreamReader::new(&mut reader);
    jpeg_stream_reader.dump()?;

    Ok(())
}
