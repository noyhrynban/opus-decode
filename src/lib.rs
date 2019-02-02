#[derive(Debug)]
enum PacketMode {
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
enum Bandwidth {
    Narrow,
    Medium,
    Wide,
    SuperWide,
    Full,
}

#[derive(Debug)]
struct FrameSize {
    pub size: f32,
}

#[derive(Debug)]
struct PacketConfiguration {
    pub mode: PacketMode,
    pub bandwidth: Bandwidth,
    pub frame_size: FrameSize,
}

#[derive(Debug)]
enum Signal {
    Mono,
    Stereo,
}

#[derive(Debug)]
enum FrameCountCode {
    Single,
    TwoEqual,
    TwoDifferent,
    Arbitrary,
}

fn opus_packet_from_toc_byte(toc_byte: u8) -> OpusPacket {
    let config_val: u8 = 0b0000_1111 & toc_byte;

    let mut mode: Option<PacketMode> = None;
    let mut bandwidth: Option<Bandwidth> = None;
    let mut frame_size: Option<f32> = None;
    match config_val {
        0...11 => {
            mode = Some(PacketMode::SILK);
            match config_val {
                0...3 => {
                    bandwidth = Some(Bandwidth::Narrow);
                    match config_val {
                        0 => frame_size = Some(10.0),
                        1 => frame_size = Some(20.0),
                        2 => frame_size = Some(40.0),
                        3 => frame_size = Some(60.0),
                        _ => (),
                    }
                }
                4...7 => {
                    bandwidth = Some(Bandwidth::Medium);
                    match config_val {
                        4 => frame_size = Some(10.0),
                        5 => frame_size = Some(20.0),
                        6 => frame_size = Some(40.0),
                        7 => frame_size = Some(60.0),
                        _ => (),
                    }
                }
                8...11 => {
                    bandwidth = Some(Bandwidth::Wide);
                    match config_val {
                        0 => frame_size = Some(10.0),
                        1 => frame_size = Some(20.0),
                        2 => frame_size = Some(40.0),
                        3 => frame_size = Some(60.0),
                        _ => (),
                    }
                }
                _ => (),
            }
        }
        12...15 => {
            mode = Some(PacketMode::Hybrid);
            match config_val {
                12...13 => {
                    bandwidth = Some(Bandwidth::SuperWide);
                    match config_val {
                        12 => frame_size = Some(10.0),
                        13 => frame_size = Some(20.0),
                        _ => (),
                    }
                }
                14...15 => {
                    bandwidth = Some(Bandwidth::Full);
                    match config_val {
                        14 => frame_size = Some(10.0),
                        15 => frame_size = Some(20.0),
                        _ => (),
                    }
                }
                _ => (),
            }
        }
        16...31 => {
            mode = Some(PacketMode::CELT);
            match config_val {
                16...19 => {
                    bandwidth = Some(Bandwidth::Narrow);
                    match config_val {
                        16 => frame_size = Some(2.5),
                        17 => frame_size = Some(5.0),
                        18 => frame_size = Some(10.0),
                        19 => frame_size = Some(20.0),
                        _ => (),
                    }
                }
                20...23 => {
                    bandwidth = Some(Bandwidth::Wide);
                    match config_val {
                        20 => frame_size = Some(2.5),
                        21 => frame_size = Some(5.0),
                        22 => frame_size = Some(10.0),
                        23 => frame_size = Some(20.0),
                        _ => (),
                    }
                }
                24...27 => {
                    bandwidth = Some(Bandwidth::SuperWide);
                    match config_val {
                        24 => frame_size = Some(2.5),
                        25 => frame_size = Some(5.0),
                        26 => frame_size = Some(10.0),
                        27 => frame_size = Some(20.0),
                        _ => (),
                    }
                }
                28...31 => {
                    bandwidth = Some(Bandwidth::Full);
                    match config_val {
                        28 => frame_size = Some(2.5),
                        29 => frame_size = Some(5.0),
                        30 => frame_size = Some(10.0),
                        31 => frame_size = Some(20.0),
                        _ => (),
                    }
                }
                _ => (),
            }
        }
        _ => (),
    }

    let config = PacketConfiguration {
        mode: mode.unwrap(),
        bandwidth: bandwidth.unwrap(),
        frame_size: FrameSize {
            size: frame_size.unwrap(),
        },
    };

    let signal_byte = (toc_byte >> 4) & 0b0001;
    let signal: Option<Signal> = match signal_byte {
        0 => Some(Signal::Mono),
        1 => Some(Signal::Stereo),
        _ => None,
    };
    let signal =
        signal.unwrap_or_else(|| panic!("Signal must be either 0 or 1. Got: {}", signal_byte));

    let code_byte = toc_byte >> 6;
    let code: Option<FrameCountCode> = match code_byte {
        0 => Some(FrameCountCode::Single),
        1 => Some(FrameCountCode::TwoEqual),
        2 => Some(FrameCountCode::TwoDifferent),
        3 => Some(FrameCountCode::Arbitrary),
        _ => None,
    };
    let code =
        code.unwrap_or_else(|| panic!("Code must be in the range 0 to 3. Got: {}", code_byte));

    OpusPacket {
        config,
        signal,
        code,
    }
}

struct OpusPacket {
    config: PacketConfiguration,
    signal: Signal,
    code: FrameCountCode,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
