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
                let opus_bytes = &ogg_packet.data;
                let p = &ogg_packet;

                // println!("\nPacket: serial 0x{:08x}, data {:08} large, first {: >5}, last {: >5}, absgp 0x{:016x}",p.stream_serial(), p.data.len(), p.first_in_page(), p.last_in_page(),p.absgp_page());

                let opus_packet = opus_decode::get_opus_packet(opus_bytes.to_vec()).unwrap();

                println!(
                    "{:?}\t{:?}\t{:?}\t{:?}\t{:?}",
                    opus_packet.config.mode,
                    opus_packet.config.bandwidth,
                    opus_packet.config.frame_size,
                    opus_packet.config.signal,
                    opus_packet.config.code
                );

                println!("{:?}", p.data);

                // next we will call some funtion that takes the Vec<u8> and returns an OPUS packet struct
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

    #[test]
    fn it_should_run_a_real_file_mono() {
        run("test_files/215-mono-10.opus".to_string()).unwrap();
    }

    #[test]
    fn it_should_run_a_real_file_12() {
        run("test_files/215-12.opus".to_string()).unwrap();
    }

    #[test]
    fn it_should_run_a_real_file_128() {
        run("test_files/215-128.opus".to_string()).unwrap();
    }
}
