
// The basic structure of this was started from
// https://github.com/RustAudio/ogg/blob/master/examples/dump-all.rs

extern crate opus_decode;

use byteorder::{LittleEndian, ReadBytesExt};
use ogg::PacketReader;
use std::env;
use std::fs::File;
use std::io::Cursor;

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

fn run(file_path: String) -> Result<(), std::io::Error> { // Begins the decoding of the Opus File
    println!("Opening file: {}", file_path);
    let mut file = File::open(file_path)?;
    let mut packet_reader = PacketReader::new(&mut file);
    let _header = packet_reader.read_packet(); // This packet should be verified to be the OpusHeader packet, at most
    let comments_header = packet_reader.read_packet(); // This is where the Metadata is stored. Maybe we'll parse this out someday.

    print_meta_data(comments_header.unwrap().unwrap().data); //Unpacks and prints the comments header of the Opus File

    let mut counter = 0; //Creates a counter used to keep track of # of packets
    loop {
        let r = packet_reader.read_packet();
        match r {
            Ok(Some(ogg_packet)) => {
                let opus_bytes = &ogg_packet.data;

                let opus_packet = opus_decode::get_opus_packet(opus_bytes.to_vec()).unwrap();

                println!(
                    "\n{:?}\t{:?}\t{:?}\t{:?}\t{:?}",
                    opus_packet.config.mode,
                    opus_packet.config.bandwidth,
                    opus_packet.config.frame_size,
                    opus_packet.config.signal,
                    opus_packet.config.code
                );

                for frame in opus_packet.frames {
                    println!("Frame bytes:\n{:?}", frame.data);
                }
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
    println!("\nFound {} packets.", counter);
    Ok(())
}

fn print_meta_data(comments_bytes: Vec<u8>) { // Takes the Comments Header previously unwrapped and unpacks and prints the data.
    struct FieldData {
        //A struct used to store the name and data of a particular comment field
        name: String,
        info: Vec<u8>,
    }

    let mut index = 8; // start at 8 to skip 'Opustags'
    let mut metadata: Vec<FieldData> = vec![]; //creates a vector of FieldData structs without specified size. There can be varying numbers of User Comments

    let mut comment_length = Cursor::new(&comments_bytes[index..index + 4])
        .read_u32::<LittleEndian>()
        .unwrap();
    index += 4;

    let data = comments_bytes.get(index..(index + comment_length as usize)); //First retrieves the Vendor String.
    let vendor = FieldData {
        name: String::from(""),
        info: Vec::from(data.unwrap()),
    };
    metadata.push(vendor);
    index += comment_length as usize;

    let number_of_comments = Cursor::new(&comments_bytes[index..(index + 4)]) //Retrieves the size of the User Comment List. 
        .read_u32::<LittleEndian>()                                           //Without this we cannot tell how many comments we need to search through.
        .unwrap();
    index += 4;

    for _ in 0..number_of_comments { //Searches an X amount of times to retrieve all User Comment data and split it into a struct that is then put into a Vec
        let element = comments_bytes.get(index..(index + 4));
        index += 4;
        comment_length = Cursor::new(element.unwrap())
            .read_u32::<LittleEndian>()
            .unwrap();
        let data = &comments_bytes[index..(index + comment_length as usize)];
        if data.is_empty() {
            break;
        }
        let split = data.iter().position(|&x| x == 0x3d).unwrap(); //Splits the data at '='
        let (left, right) = data.split_at(split);
        let title = std::str::from_utf8(left).unwrap(); //Converts data into String
        let info = (&right[1..right.len()]).to_vec(); //Data remains as a u8 and removes the '=' from info.
        let information = FieldData {
            name: String::from(title),
            info: info,
        };
        metadata.push(information); //Pushes FieldData struct information into metadata vec
        index += comment_length as usize;
    }
    for item in metadata { //Prints the data from every struct inside of the metadata vec.
        println!(
            "{}: {}",
            item.name,
            std::str::from_utf8(&item.info.to_vec()).unwrap() //Converts u8 from info into string.
        );
    }
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
    fn it_should_run_a_real_file_2_frames_per_packet() {
        run("test_files/tone-40ms.opus".to_string()).unwrap();
    }

    #[test]
    fn it_should_run_a_real_file_more_than_2_frames_per_packet() {
        run("test_files/tone-60ms.opus".to_string()).unwrap();
    }

    #[test]
    fn it_should_run_a_real_file_more_than_2_frames_per_packet_and_lots_of_padding() {
        run("test_files/silence-60ms-1000.opus".to_string()).unwrap();
    } 
}
