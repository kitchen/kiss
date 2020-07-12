use nom::{map_opt, named, tag, take};
extern crate num;
extern crate num_derive;
use num_derive::FromPrimitive;

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
enum KissFrame {
    // Data(&[u8]),
    Command(FrameType),
}

pub const FEND: u8 = 0xC0;
pub const FESC: u8 = 0xDB;
pub const TFEND: u8 = 0xDC;
pub const TFESC: u8 = 0xDD;

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromPrimitive, PartialEq)]
#[allow(dead_code)]
pub enum FrameType {
    DataFrame = 0x00,
    TXDelay = 0x01,
    P = 0x02,
    SlotTime = 0x03,
    TXTail = 0x04,
    FullDuplex = 0x05,
    SetHardware = 0x06,
    Return = 0xFF,
}

named!(pub fend, tag!([FEND]));
named!(pub tfend, tag!([TFEND]));
named!(pub fesc, tag!([FESC]));
named!(pub tfesc, tag!([TFESC]));
named!(
    pub frame_type(&[u8]) -> FrameType,
    map_opt!(
        take!(1),
        |bytes: &[u8]| {
            num::FromPrimitive::from_u8(bytes[0])
        }
    )
);

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    use super::*;
    const EMPTY: &[u8] = &[];

    #[test]
    fn test_fend_fesc() {
        assert_eq!(Ok((EMPTY, &[FEND][..])), fend(&[FEND]));
        assert_eq!(Ok((EMPTY, &[TFEND][..])), tfend(&[TFEND]));
        assert_eq!(Ok((EMPTY, &[FESC][..])), fesc(&[FESC]));
        assert_eq!(Ok((EMPTY, &[TFESC][..])), tfesc(&[TFESC]));
    }

    #[test]
    fn test_frame_type() {
        assert_eq!(
            Ok((EMPTY, FrameType::DataFrame)),
            frame_type(&[FrameType::DataFrame as u8])
        );
        assert_eq!(
            Ok((EMPTY, FrameType::TXDelay)),
            frame_type(&[FrameType::TXDelay as u8])
        );
        assert_eq!(Ok((EMPTY, FrameType::P)), frame_type(&[FrameType::P as u8]));
        assert_eq!(
            Ok((EMPTY, FrameType::SlotTime)),
            frame_type(&[FrameType::SlotTime as u8])
        );
        assert_eq!(
            Ok((EMPTY, FrameType::TXTail)),
            frame_type(&[FrameType::TXTail as u8])
        );
        assert_eq!(
            Ok((EMPTY, FrameType::FullDuplex)),
            frame_type(&[FrameType::FullDuplex as u8])
        );
        assert_eq!(
            Ok((EMPTY, FrameType::SetHardware)),
            frame_type(&[FrameType::SetHardware as u8])
        );
        assert_eq!(
            Ok((EMPTY, FrameType::Return)),
            frame_type(&[FrameType::Return as u8])
        );
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    #[ignore]
    fn send_the_characters_test_out_of_tnc_port_0() {
        let frame_data: [u8; 7] = [0xC0, 0x00, 0x54, 0x45, 0x53, 0x54, 0xC0];

        todo!();
    }

    #[test]
    #[ignore]
    fn send_the_characters_hello_out_of_tnc_port_5() {
        let frame_data: [u8; 8] = [0xC0, 0x50, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xC0];

        todo!();
    }

    #[test]
    #[ignore]
    fn send_some_bytes_out_of_tnc_port_0() {
        let frame_data: [u8; 7] = [0xC0, 0x00, 0xDB, 0xDC, 0xDB, 0xDD, 0xC0];
        todo!();
    }

    #[test]
    #[ignore]
    fn exit_kiss_mode() {
        let frame_data: [u8; 3] = [0xC0, 0xFF, 0xC0];

        todo!();
    }
}
