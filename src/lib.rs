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

    let mode: PacketMode;
    let bandwidth: Bandwidth;
    let frame_size: f32;
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

    let signal: Signal = match (toc_byte & 0b100) >> 2 {
        0 => Signal::Mono,
        1 => Signal::Stereo,
        _ => return Err("match_code_byte_failed impossibly"),
    };

    let code: FrameCountCode = match toc_byte & 0b0000_0011 {
        0 => FrameCountCode::Single,
        1 => FrameCountCode::TwoEqual,
        2 => FrameCountCode::TwoDifferent,
        3 => FrameCountCode::Arbitrary,
        _ => return Err("match_code_byte_failed impossibly"),
    };

    Ok(PacketConfiguration {
        mode,
        bandwidth,
        frame_size,
        signal,
        code,
    })
}

pub fn get_opus_packet(packet_data: Vec<u8>) -> Result<OpusPacket, &'static str> {
    if let Some((toc_byte, data)) = packet_data.split_first() {
        let config = packet_config_from_toc_byte(*toc_byte).unwrap();
        let data = data.to_vec();
        let frames = vec![];
        match config.code {
            FrameCountCode::Single => vec![Frame { data }], // code 0
            FrameCountCode::TwoEqual => {
                // code 1
                println!("Data length: {}", data.len());
                let (one, two) = data.split_at(data.len() / 2);
                vec![Frame { data: one.to_vec() }, Frame { data: two.to_vec() }]
            }
            FrameCountCode::TwoDifferent => {
                // code 2
                let (size, mut data) = data.split_first().unwrap();
                println!("size: {}, data len: {}", size, data.len());
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
                vec![]
            }
        };

        Ok(OpusPacket {
            config,
            frames,
        })
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
