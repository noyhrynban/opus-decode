#[derive(Debug)]
pub enum PacketMode {
    SILK,   // low bandwidth algorithm
    Hybrid, // both SILK and CELT
    CELT,   // high bandwidth algorithm
}

// Taken from https://tools.ietf.org/html/rfc6716#section-2
// +----------------------+-----------------+-------------------------+
// | Abbreviation         | Audio Bandwidth | Sample Rate (Effective) |
// +----------------------+-----------------+-------------------------+
// | NB (narrowband)      |           4 kHz |                   8 kHz |
// |                      |                 |                         |
// | MB (medium-band)     |           6 kHz |                  12 kHz |
// |                      |                 |                         |
// | WB (wideband)        |           8 kHz |                  16 kHz |
// |                      |                 |                         |
// | SWB (super-wideband) |          12 kHz |                  24 kHz |
// |                      |                 |                         |
// | FB (fullband)        |      20 kHz (*) |                  48 kHz |
// +----------------------+-----------------+-------------------------+
#[derive(Debug)]
pub enum Bandwidth {
    Narrow,
    Medium,
    Wide,
    SuperWide,
    Full,
}

#[derive(Debug, PartialEq)]
pub enum Signal {
    Mono,
    Stereo,
}

#[derive(Debug, PartialEq)]
pub enum FrameCountCode {
    Single,
    TwoEqual,
    TwoDifferent,
    Arbitrary,
}

pub struct Frame {
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct PacketConfiguration {
    pub mode: PacketMode,
    pub bandwidth: Bandwidth,
    pub frame_size: f32,
    pub signal: Signal,
    pub code: FrameCountCode,
}

pub struct OpusPacket {
    pub config: PacketConfiguration,
    pub frames: Vec<Frame>,
}

pub fn packet_config_from_toc_byte(toc_byte: u8) -> Result<PacketConfiguration, &'static str> {
    let config_val: u8 = toc_byte >> 3;

    let bandwidth: Bandwidth;
    let code: FrameCountCode;
    let frame_size: f32;
    let mode: PacketMode;
    let signal: Signal;
    match config_val {
        0...11 => {
            mode = PacketMode::SILK;
            match config_val {
                0...3 => {
                    bandwidth = Bandwidth::Narrow;
                    match config_val {
                        0 => frame_size = 10.0,
                        1 => frame_size = 20.0,
                        2 => frame_size = 40.0,
                        3 => frame_size = 60.0,
                        _ => unimplemented!("match_code_byte_failed impossibly"),
                    }
                }
                4...7 => {
                    bandwidth = Bandwidth::Medium;
                    match config_val {
                        4 => frame_size = 10.0,
                        5 => frame_size = 20.0,
                        6 => frame_size = 40.0,
                        7 => frame_size = 60.0,
                        _ => unimplemented!("match_code_byte_failed impossibly"),
                    }
                }
                8...11 => {
                    bandwidth = Bandwidth::Wide;
                    match config_val {
                        8 => frame_size = 10.0,
                        9 => frame_size = 20.0,
                        10 => frame_size = 40.0,
                        11 => frame_size = 60.0,
                        _ => unimplemented!("match_code_byte_failed impossibly"),
                    }
                }
                _ => unimplemented!("match_code_byte_failed impossibly"),
            }
        }
        12...15 => {
            mode = PacketMode::Hybrid;
            match config_val {
                12...13 => {
                    bandwidth = Bandwidth::SuperWide;
                    match config_val {
                        12 => frame_size = 10.0,
                        13 => frame_size = 20.0,
                        _ => unimplemented!("match_code_byte_failed impossibly"),
                    }
                }
                14...15 => {
                    bandwidth = Bandwidth::Full;
                    match config_val {
                        14 => frame_size = 10.0,
                        15 => frame_size = 20.0,
                        _ => unimplemented!("match_code_byte_failed impossibly"),
                    }
                }
                _ => unimplemented!("match_code_byte_failed impossibly"),
            }
        }
        16...31 => {
            mode = PacketMode::CELT;
            match config_val {
                16...19 => {
                    bandwidth = Bandwidth::Narrow;
                    match config_val {
                        16 => frame_size = 2.5,
                        17 => frame_size = 5.0,
                        18 => frame_size = 10.0,
                        19 => frame_size = 20.0,
                        _ => unimplemented!("match_code_byte_failed impossibly"),
                    }
                }
                20...23 => {
                    bandwidth = Bandwidth::Wide;
                    match config_val {
                        20 => frame_size = 2.5,
                        21 => frame_size = 5.0,
                        22 => frame_size = 10.0,
                        23 => frame_size = 20.0,
                        _ => unimplemented!("match_code_byte_failed impossibly"),
                    }
                }
                24...27 => {
                    bandwidth = Bandwidth::SuperWide;
                    match config_val {
                        24 => frame_size = 2.5,
                        25 => frame_size = 5.0,
                        26 => frame_size = 10.0,
                        27 => frame_size = 20.0,
                        _ => unimplemented!("match_code_byte_failed impossibly"),
                    }
                }
                28...31 => {
                    bandwidth = Bandwidth::Full;
                    match config_val {
                        28 => frame_size = 2.5,
                        29 => frame_size = 5.0,
                        30 => frame_size = 10.0,
                        31 => frame_size = 20.0,
                        _ => unimplemented!("match_code_byte_failed impossibly"),
                    }
                }
                _ => unimplemented!("match_code_byte_failed impossibly"),
            }
        }
        _ => unimplemented!("match_code_byte_failed impossibly"),
    }

    match (toc_byte & 0b100) >> 2 {
        0 => signal = Signal::Mono,
        1 => signal = Signal::Stereo,
        _ => unimplemented!("match_code_byte_failed impossibly"),
    };

    match toc_byte & 0b0000_0011 {
        0 => code = FrameCountCode::Single,
        1 => code = FrameCountCode::TwoEqual,
        2 => code = FrameCountCode::TwoDifferent,
        3 => code = FrameCountCode::Arbitrary,
        _ => unimplemented!("match_code_byte_failed impossibly"),
    };

    Ok(PacketConfiguration {
        bandwidth,
        code,
        frame_size,
        mode,
        signal,
    })
}

pub fn get_opus_packet(packet_data: Vec<u8>) -> Result<OpusPacket, &'static str> {
    if let Some((toc_byte, data)) = packet_data.split_first() {
        let config = packet_config_from_toc_byte(*toc_byte).unwrap();
        let data = data.to_vec();
        let frames = match config.code {
            FrameCountCode::Single => vec![Frame { data }], // code 0
            FrameCountCode::TwoEqual => {
                // code 1
                assert!(data.len() % 2 == 0);
                let (one, two) = data.split_at(data.len() / 2);
                vec![Frame { data: one.to_vec() }, Frame { data: two.to_vec() }]
            }
            FrameCountCode::TwoDifferent => {
                // code 2
                let (size, mut data) = data.split_first().unwrap();
                let mut size = usize::from(*size);
                if size > 251 {
                    let tuple = data.split_first().unwrap();
                    data = tuple.1;
                    size += usize::from(*tuple.0) * 4;
                }

                let (one, two) = data.split_at(size);
                vec![Frame { data: one.to_vec() }, Frame { data: two.to_vec() }]
            }
            FrameCountCode::Arbitrary => {
                // code 3
                let (frame_count_byte, mut data) = data.split_first().unwrap();
                let frame_count_byte = *frame_count_byte;
                let frame_count = frame_count_byte & 0b0011_1111;
                let padded = (frame_count_byte >> 6) & 0b0000_0001 == 1;
                let vbr = (frame_count_byte >> 7) & 0b0000_0001 == 1;

                if padded {
                    // remove the padding at the end of the packet
                    let (mut first, mut new_data_window) = data.split_first().unwrap();
                    data = new_data_window;
                    let mut total_padding_length: usize = 0;
                    let mut padding_length = *first;
                    while padding_length == 255 {
                        total_padding_length += 254;
                        let tuple = new_data_window.split_first().unwrap();
                        first = tuple.0;
                        new_data_window = tuple.1;
                        padding_length = *first;
                        data = new_data_window;
                    }
                    total_padding_length += usize::from(padding_length);
                    let (frame_data, _padding) = data.split_at(data.len() - total_padding_length);
                    data = frame_data;
                }

                if vbr {
                    // VBR frames
                    let mut frame_sizes: Vec<usize> = vec![];
                    let mut frames: Vec<Frame> = vec![];
                    for _ in 0..(frame_count - 1) {
                        let (size_byte, mut data_window) = data.split_first().unwrap();
                        let mut frame_size: usize = usize::from(*size_byte);
                        if *size_byte == 255 {
                            let tuple = data_window.split_first().unwrap();
                            frame_size += usize::from(*tuple.0);
                            data_window = tuple.1;
                        }
                        frame_sizes.push(frame_size);
                        data = data_window;
                    }
                    for frame_number in 0..frame_count - 1 {
                        let tuple = data.split_at(frame_sizes[usize::from(frame_number)]);
                        frames.push(Frame {
                            data: tuple.0.to_vec(),
                        });
                        data = tuple.1;
                    }
                    frames.push(Frame {
                        data: data.to_vec(),
                    });
                    assert_eq!(usize::from(frame_count), frames.len());
                    frames
                } else {
                    // CBR frames
                    assert!((data.len() % usize::from(frame_count)) == 0);
                    let framesize = data.len() / usize::from(frame_count);
                    data.chunks(framesize)
                        .map(|chunk| Frame {
                            data: chunk.to_vec(),
                        })
                        .collect()
                }
            }
        };

        Ok(OpusPacket { config, frames })
    } else {
        Err("splitting the packet into a TOC byte and data failed")
    }
}

mod tests {

    #[test]
    fn it_should_recognize_stereo() {
        let config = super::packet_config_from_toc_byte(0b0000_0100).unwrap();
        assert_eq!(config.signal, super::Signal::Stereo);
    }

    #[test]
    fn it_should_recognize_code_1() {
        let config = super::packet_config_from_toc_byte(0b0000_0001).unwrap();
        assert_eq!(config.code, super::FrameCountCode::TwoEqual);
    }

    #[test]
    fn it_should_recognize_code_2() {
        let config = super::packet_config_from_toc_byte(0b0000_0010).unwrap();
        assert_eq!(config.code, super::FrameCountCode::TwoDifferent);
    }

    #[test]
    fn it_should_recognize_code_3() {
        let config = super::packet_config_from_toc_byte(0b0000_0011).unwrap();
        assert_eq!(config.code, super::FrameCountCode::Arbitrary);
    }

    #[test]
    fn it_creates_packet_config_from_u8() {
        for byte in 0..255 {
            let config = super::packet_config_from_toc_byte(byte);
            assert!(config.is_ok());
        }
    }

    #[test]
    fn it_should_create_opus_packet() {
        let bytes = vec![0, 0];
        assert_eq!(true, super::get_opus_packet(bytes).is_ok())
    }

    #[test]
    #[should_panic]
    fn it_creates_opus_packet_from_vec_u8() {
        super::get_opus_packet([].to_vec()).unwrap();
    }
}
