// The basic structure of this was started from
// https://github.com/RustAudio/ogg/blob/master/examples/dump-all.rs

extern crate opus_decode;

use byteorder::{LittleEndian, ReadBytesExt};
use ogg::PacketReader;
use std::env;
use std::fs::File;
use std::io::Cursor;

struct AlbumInfo {
    //Struct for storing the metadata per file
    track: String,
    artist: String,
    album: String,
}

struct FieldData {
    name: String,
    info: Vec<u8>,
}

#[cfg_attr(tarpaulin, skip)]
fn main() {
    let file_path = env::args()
        .nth(1)
        .expect("No arg found. Please specify a file to open.");
    match run(file_path) {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    }
}

fn run(file_path: String) -> Result<(), std::io::Error> {
    println!("Opening file: {}", file_path);
    let mut file = File::open(file_path)?;
    let mut packet_reader = PacketReader::new(&mut file);
    let _header = packet_reader.read_packet(); // This packet should be verified to be the OpusHeader packet, at most
    let _comments_header = packet_reader.read_packet(); // This is where the Metadata is stored. Maybe we'll parse this out someday.

    let mut meta = AlbumInfo {
        track: String::from(" "),
        artist: String::from(" "),
        album: String::from(" "),
    };

    let comment_bytes = _comments_header.unwrap().unwrap().data;
    let mut index = 8;
    let mut metadata: Vec<FieldData> = vec![];

    let mut element = comment_bytes.get(index..index + 4);
    index += 4;
    let length = Cursor::new(element.unwrap())
        .read_u32::<LittleEndian>()
        .unwrap();
    let mut data = comment_bytes.get(index..(index + length as usize));
    let vendor = FieldData {
        name: String::from(""),
        info: Vec::from(data.unwrap()),
    };
    metadata.push(vendor);

    index = index + length as usize + 1;

    element = comment_bytes.get(index..index + 4);
    let length = Cursor::new(element.unwrap())
        .read_u32::<LittleEndian>()
        .unwrap();

    let list = length;
    index += 4;
    for comment in 0..list {
        element = comment_bytes.get(index..index + 4);
        index += 4;
        let length = Cursor::new(element.unwrap())
            .read_u32::<LittleEndian>()
            .unwrap();
        data = comment_bytes.get(index..(index + length as usize));
        let (left, right) = data.unwrap().split_at(0x3d);
        let title = std::str::from_utf8(left).unwrap();
        let information = FieldData {
            name: String::from(title),
            info: Vec::from(right),
        };
        metadata.push(information);
        index = index + length as usize + 1;
    }
    for item in metadata{
        println!("name: {}", item.name);
    }

    let mut counter = 0;
    loop {
        let r = packet_reader.read_packet();
        match r {
            Ok(Some(ogg_packet)) => {
                let opus_bytes = ogg_packet.data;
                // next we will call some funtion that takes the Vec<u8> and returns an OPUS packet struct
                let _opus_packet = opus_decode::get_opus_packet(opus_bytes);
            }
            // End of stream
            Ok(None) => break,
            Err(e) => {
                println!("Encountered Error: {:?}", e);
                break;
            }
        }
        counter += 1;
    }
    println!("Found {} packets.", counter);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    #[should_panic]
    fn it_should_fail_to_run_with_no_file() {
        run("".to_string()).unwrap();
    }

    #[test]
    fn it_should_run_but_fail() {
        run("test_files/corrupt.opus".to_string()).unwrap();
    }

    #[test]
    fn it_should_run() {
        run("test_files/tiny.opus".to_string()).unwrap();
    }
}
