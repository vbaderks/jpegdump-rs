use structopt::StructOpt;
use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

#[allow(dead_code)]


enum JpegMarker {
    Restart0 = 0xD0,                   // RST0
    Restart1 = 0xD1,                   // RST1
    Restart2 = 0xD2,                   // RST2
    Restart3 = 0xD3,                   // RST3
    Restart4 = 0xD4,                   // RST4
    Restart5 = 0xD5,                   // RST5
    Restart6 = 0xD6,                   // RST6
    Restart7 = 0xD7,                   // RST7
    StartOfImage = 0xD8,               // SOI
    EndOfImage = 0xD9,                 // EOI
    StartOfScan = 0xDA,                // SOS
    DefineRestartInterval = 0xDD,      // DRI
    StartOfFrameJpegLS = 0xF7,         // SOF_55: Marks the start of a (JPEG-LS) encoded frame.
    JpegLSExtendedParameters = 0xF8,   // LSE: JPEG-LS extended parameters.
    ApplicationData0 = 0xE0,           // APP0: Application data 0: used for JFIF header.
    ApplicationData7 = 0xE7,           // APP7: Application data 7: color space.
    ApplicationData8 = 0xE8,           // APP8: Application data 8: colorXForm.
    ApplicationData14 = 0xEE,          // APP14: Application data 14: used by Adobe
    Comment = 0xFE                     // COM:  Comment block.
}


pub struct JpegStreamReader<'a> {
    buf_reader: & 'a mut ( dyn Read + 'a),
    jpegls_stream: bool,
    position: i32
}

impl<'a> JpegStreamReader<'a> {
    pub fn new(the_buf_reader: &'a mut dyn Read) -> JpegStreamReader<'a> {
        Self { buf_reader: the_buf_reader, jpegls_stream: false, position: 0 }
    }

    pub fn dump(&mut self) -> Result<(), io::Error> {
        let mut value= self.read_byte_safe()?;
        while value != -1 {
            if value == 0xFF {
                let marker_code = self.read_byte_safe()?;
                if self.is_marker_code(marker_code) {
                    self.dump_marker(marker_code);
                    if marker_code == JpegMarker::StartOfFrameJpegLS as i32 {
                        self.jpegls_stream = true;
                    }
                }
            }
            value= self.read_byte_safe()?
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

    fn dump_marker(&mut self, marker_code: i32) {
        match marker_code {
            x if x == JpegMarker::StartOfImage as i32 => self.dump_start_of_image(),
            x if x == JpegMarker::StartOfFrameJpegLS as i32 => self.dump_start_of_frame_jpegls(),
            x if x == JpegMarker::StartOfScan as i32 => self.dump_start_of_scan(),
            x if x == JpegMarker::EndOfImage as i32 => self.dump_end_of_image(),
            x if x == JpegMarker::JpegLSExtendedParameters as i32 => self.dump_jpegls_extended_parameters(),
            _ => println!("{:>8} Marker 0xFF{:X}", self.get_start_offset(), marker_code)
        }
    }

    fn get_start_offset(&self) -> i32 {
        return self.get_position() - 2;
    }

    fn get_position(&self) -> i32 {
        return self.position;
    }

    fn read_byte_safe(&mut self) -> Result<i32, io::Error> {
        let mut buffer = [0; 1];

        let n = self.buf_reader.read(&mut buffer[..])?;
        if n == 0 {
            return Ok(-1);
        }

        self.position += 1;

        Ok(buffer[0] as i32)
    }

    fn read_byte(&mut self) -> i32 {
        let mut buffer = [0; 1];

        let n = self.buf_reader.read(&mut buffer[..]).unwrap();
        if n == 0 {
            return 0;
        }

        self.position += 1;

        buffer[0] as i32
    }

    fn read_u16_big_endian(&mut self) -> u16{
        let mut buffer = [0; 2];

        let n = self.buf_reader.read(&mut buffer[..]).unwrap();
        if n == 0 {
            return 0;
        }

        self.position += 2;

        ((buffer[0] as u16) << 8) | (buffer[1] as u16)
    }

    fn dump_start_of_image(&self) {
        println!("{:>8} Marker 0xFFD8: SOI (Start Of Image), defined in ITU T.81/IEC 10918-1", self.get_start_offset())
    }

    fn dump_end_of_image(&self) {
        println!("{:>8} Marker 0xFFD9: EOI (End Of Image), defined in ITU T.81/IEC 10918-1", self.get_start_offset())
    }

    fn dump_start_of_frame_jpegls(&mut self) {
        println!("{:>8} Marker 0xFFF7: SOF_55 (Start Of Frame JPEG-LS), defined in ITU T.87/IEC 14495-1 JPEG LS", self.get_start_offset());
        println!("{:>8}  Size = {}", self.get_position(), self.read_u16_big_endian());
        println!("{:>8}  Sample precision (P) = {}", self.get_position(), self.read_byte());
        println!("{:>8}  Number of lines (Y) = {}", self.get_position(), self.read_u16_big_endian());
        println!("{:>8}  Number of samples per line (X) = {}", self.get_position(), self.read_u16_big_endian());
        let position = self.get_position();
        let component_count = self.read_byte();
        println!("{:>8}  Number of image components in a frame (Nf) = {1}", position, component_count);

        for _ in 0..component_count {
            println!("{:>8}   Component identifier (Ci) = {}", self.get_position(), self.read_byte());
            let position = self.get_position();
            let sampling_factor = self.read_byte();
            println!("{:>8}   H and V sampling factor (Hi + Vi) = {} ({} + {})", position, sampling_factor, sampling_factor >> 4, sampling_factor & 0xF);
            println!("{:>8}   Quantization table (Tqi) [reserved, should be 0] = {}", self.get_position(), self.read_byte());
        }
    }

    fn dump_start_of_scan(&mut self) {
        println!("{:>8} Marker 0xFFDA: SOS (Start Of Scan), defined in ITU T.81/IEC 10918-1", self.get_start_offset());
        println!("{:>8}  Size = {}", self.get_position(), self.read_u16_big_endian());
        let component_count = self.read_byte();
        println!("{:>8}  Component Count = {}", self.get_position(), component_count);
        for _ in 0..component_count {
            println!("{:>8}   Component identifier (Ci) = {}", self.get_position(), self.read_byte());
            let mapping_table_selector = self.read_byte();
            println!("{:>8}   Mapping table selector = {} {}", self.get_position(), mapping_table_selector, if mapping_table_selector == 0 {"(None)"} else {""});
        }

        println!("{:>8}  Near lossless (NEAR parameter) = {}", self.get_position(), self.read_byte());
        let interleave_mode = self.read_byte();
        println!("{:>8}  Interleave mode (ILV parameter) = {} ({})", self.get_position(), interleave_mode, JpegStreamReader::get_interleave_mode_name(interleave_mode));
        println!("{:>8}  Point Transform = {}", self.get_position(), self.read_byte());
    }

    fn dump_jpegls_extended_parameters(&mut self)
    {
        println!("{:>8} Marker 0xFFF8: LSE (JPEG-LS ), defined in ITU T.87/IEC 14495-1 JPEG LS", self.get_start_offset());
        println!("{:>8}  Size = {}", self.get_position(), self.read_u16_big_endian());
        let ep_type = self.read_byte();

        print!("{:>8}  Type = {}", self.get_position(), ep_type);
        match ep_type {
            1 => {
                println!(" (Preset coding parameters)");
                println!("{:>8}  MaximumSampleValue = {1}", self.get_position(), self.read_u16_big_endian());
                println!("{:>8}  Threshold 1 = {1}", self.get_position(), self.read_u16_big_endian());
                println!("{:>8}  Threshold 2 = {1}", self.get_position(), self.read_u16_big_endian());
                println!("{:>8}  Threshold 3 = {1}", self.get_position(), self.read_u16_big_endian());
                println!("{:>8}  Reset value = {1}", self.get_position(), self.read_u16_big_endian());
            }
            _ =>  println!(" (Unknown")
        }
    }

    fn get_interleave_mode_name(interleave_mode: i32) -> &'static str {
        match interleave_mode {
            0 => "None",
            1 => "Line",
            2 => "Sample",
            _ => "Unknown"
        }
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

    let mut reader = BufReader::new(File::open(args.path)?);
    let mut jpeg_stream_reader = JpegStreamReader::new(&mut reader);

    jpeg_stream_reader.dump()?;

    Ok(())
}
