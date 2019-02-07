extern crate opus_decode;

#[cfg(test)]
mod tests {
    use opus_decode::get_opus_packet;
    use opus_decode::packet_config_from_toc_byte;

    #[test]
    fn it_creates_packet_config_from_u8() {
        for byte in 0..255 {
            println!("Byte is: {}", byte);
            let config = packet_config_from_toc_byte(byte);
            assert!(config.is_ok());
        }
    }

    #[test]
    fn it_creates_opus_packet_from_vec_u8() {
        get_opus_packet([].to_vec()); // Don't care about the result since it should fail
    }
}
