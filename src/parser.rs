use nom::combinator::rest;
use nom::{call, delimited, do_parse, map_opt, named, switch, tag, take};
extern crate num;
extern crate num_derive;
use num_derive::FromPrimitive;

pub struct Frame {
    pub port: u8,
    pub payload: Payload,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Payload {
    Data(Vec<u8>),
    TXDelay(u8),
    P(u8),
    SlotTime(u8),
    TXTail(u8),
    FullDuplex(bool),
    SetHardware(Vec<u8>),
    Return,
}

pub const FEND: u8 = 0xC0;
pub const FESC: u8 = 0xDB;
pub const TFEND: u8 = 0xDC;
pub const TFESC: u8 = 0xDD;

#[repr(u8)]
#[derive(Debug, FromPrimitive, PartialEq)]
pub enum FrameType {
    Data = 0x00,
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

named!(pub data_frame(&[u8]) -> Payload,
       do_parse!(
           data: rest >>
               (Payload::Data(data.to_vec()))
       )
);

named!(pub txdelay_frame(&[u8]) -> Payload,
       do_parse!(
           txdelay: take!(1) >>
               (Payload::TXDelay(txdelay[0]))
       )
);

named!(pub p_frame(&[u8]) -> Payload,
       do_parse!(
           p: take!(1) >>
               (Payload::P(p[0]))
       )
);

named!(pub slot_time_frame(&[u8]) -> Payload,
       do_parse!(
           slot_time: take!(1) >>
               (Payload::SlotTime(slot_time[0]))
       )
);

named!(pub txtail_frame(&[u8]) -> Payload,
       do_parse!(
           txtail: take!(1) >>
               (Payload::TXTail(txtail[0]))
       )
);

named!(pub fullduplex_frame(&[u8]) -> Payload,
       do_parse!(
           fullduplex: take!(1) >>
               (Payload::FullDuplex(fullduplex[0] != 0))
       )
);

named!(pub sethardware_frame(&[u8]) -> Payload,
       do_parse!(
           data: rest >>
               (Payload::SetHardware(data.to_vec()))
       )
);

named!(pub return_frame(&[u8]) -> Payload,
       do_parse!(
           (Payload::Return)
       )
);

named!(pub frame_payload(&[u8]) -> Payload,
       switch!(call!(frame_type),
               FrameType::Data => call!(data_frame) |
               FrameType::TXDelay => call!(txdelay_frame) |
               FrameType::P => call!(p_frame) |
               FrameType::SlotTime => call!(slot_time_frame) |
               FrameType::TXTail => call!(txtail_frame) |
               FrameType::FullDuplex => call!(fullduplex_frame) |
               FrameType::SetHardware => call!(sethardware_frame) |
               FrameType::Return => call!(return_frame)
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
            Ok((EMPTY, FrameType::Data)),
            frame_type(&[FrameType::Data as u8])
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
    fn test_txdelay_frame() {
        let data = &[42];
        assert_eq!(Ok((EMPTY, Payload::TXDelay(42))), txdelay_frame(data));
    }

    #[test]
    fn test_data_frame() {
        let data = &[42, 43];
        assert_eq!(Ok((EMPTY, Payload::Data(vec![42, 43]))), data_frame(data));
    }

    #[test]
    fn test_p_frame() {
        let data = &[42];
        assert_eq!(Ok((EMPTY, Payload::P(42))), p_frame(data));
    }

    #[test]
    fn test_slot_time_frame() {
        let data = &[42];
        assert_eq!(Ok((EMPTY, Payload::SlotTime(42))), slot_time_frame(data));
    }

    #[test]
    fn test_txtail_frame() {
        let data = &[42];
        assert_eq!(Ok((EMPTY, Payload::TXTail(42))), txtail_frame(data));
    }

    #[test]
    fn test_fullduplex_frame() {
        let data = &[0];
        assert_eq!(
            Ok((EMPTY, Payload::FullDuplex(false))),
            fullduplex_frame(data)
        );

        let data = &[42];
        assert_eq!(
            Ok((EMPTY, Payload::FullDuplex(true))),
            fullduplex_frame(data)
        );
    }

    #[test]
    fn test_sethardware_frame() {
        let data = &[42, 43];
        assert_eq!(
            Ok((EMPTY, Payload::SetHardware(vec![42, 43]))),
            sethardware_frame(data)
        );
    }

    #[test]
    fn test_return_frame() {
        assert_eq!(Ok((EMPTY, Payload::Return)), return_frame(EMPTY));
    }

    #[test]
    fn test_frame_payload() {
        let data = &[FrameType::Data as u8, 42, 43];
        assert_eq!(
            Ok((EMPTY, Payload::Data(vec![42, 43]))),
            frame_payload(data)
        );

        let data = &[FrameType::TXDelay as u8, 42];
        assert_eq!(Ok((EMPTY, Payload::TXDelay(42))), frame_payload(data));

        let data = &[FrameType::P as u8, 42];
        assert_eq!(Ok((EMPTY, Payload::P(42))), frame_payload(data));

        let data = &[FrameType::SlotTime as u8, 42];
        assert_eq!(Ok((EMPTY, Payload::SlotTime(42))), frame_payload(data));

        let data = &[FrameType::TXTail as u8, 42];
        assert_eq!(Ok((EMPTY, Payload::TXTail(42))), frame_payload(data));

        let data = &[FrameType::FullDuplex as u8, 0];
        assert_eq!(Ok((EMPTY, Payload::FullDuplex(false))), frame_payload(data));

        let data = &[FrameType::FullDuplex as u8, 42];
        assert_eq!(Ok((EMPTY, Payload::FullDuplex(true))), frame_payload(data));

        let data = &[FrameType::SetHardware as u8, 42, 43];
        assert_eq!(
            Ok((EMPTY, Payload::SetHardware(vec![42, 43]))),
            frame_payload(data)
        );

        let data = &[FrameType::Return as u8];
        assert_eq!(Ok((EMPTY, Payload::Return)), frame_payload(data));
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

    // #[test]
    // fn exit_kiss_mode() {
    //     let data = &[FEND, FrameType::Return as u8, FEND];
    //     assert_eq!(Ok((EMPTY, Payload::Return)), frame(data));
    // }
}
