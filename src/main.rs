// The basic structure of this was started from
// https://github.com/RustAudio/ogg/blob/master/examples/dump-all.rs

extern crate opus_decode;

use ogg::PacketReader;
use std::env;
use std::fs::File;

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
    let _comments_header = packet_reader.read_packet(); // This is where the Metadata is stored. Maye we'll parse this out someday.

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
    fn it_should_run() {
        run("test_files/tiny.opus".to_string()).unwrap();
    }
}
