#[cfg(test)]
mod tests {
    use crate::derive::{Config, UsbDerive};
    use anyhow::Result;
    use starcoin_consensus::Consensus;
    use std::convert::TryInto;

    const INPUT_DATA: [u8; 76] = [
        0x05, 0x05, 0xc0, 0xa7, 0xdb, 0xc7, 0x05, 0xb0, 0xad, 0xf8, 0x2c, 0x58, 0x1a, 0xae, 0xe4,
        0x8b, 0x2e, 0x0a, 0xee, 0x2e, 0xa8, 0x97, 0x2d, 0xd7, 0x9d, 0xba, 0xf3, 0xca, 0x28, 0xac,
        0xca, 0x5f, 0x73, 0xca, 0x2a, 0x90, 0x9c, 0x8c, 0x24, 0xf7, 0x09, 0x00, 0x80, 0xf9, 0x87,
        0x13, 0xc6, 0x91, 0x9a, 0x42, 0x38, 0x9d, 0x53, 0xcb, 0xde, 0xd0, 0x4d, 0x02, 0x6c, 0x1d,
        0xe4, 0x25, 0xf8, 0x77, 0xe8, 0x70, 0xb3, 0x8f, 0x91, 0x4c, 0xef, 0x40, 0xc6, 0x7f, 0xa4,
        0x00,
    ];

    fn setup(path: &str) -> Result<UsbDerive> {
        let mut derive = UsbDerive::open(path, Config::default()).expect("Must open serial port");
        derive.set_hw_params()?;
        derive.set_opcode()?;
        Ok(derive)
    }

    #[test]
    fn test_detect() {
        let derive_info = UsbDerive::detect(1155, 22336).unwrap();
        let mut derive = setup(&derive_info[0].port_name).unwrap();
        let state = derive.get_state().unwrap();
        println!("{:?}", state);
    }
}
