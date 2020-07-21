use nom::branch::alt;
use nom::bytes::streaming::{tag, take, take_till};
use nom::combinator::{map_opt, map_parser, rest};
use nom::multi::fold_many0;
use nom::number::streaming::be_u8;
use nom::sequence::delimited;
use nom::{do_parse, named, take, IResult};
extern crate num;
extern crate num_derive;
use num_derive::FromPrimitive;

#[derive(Debug, PartialEq)]
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
    Return = 0x0F,
}

pub const FEND: u8 = 0xC0;
pub fn fend(i: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(&[FEND])(i)
}

pub const FESC: u8 = 0xDB;
pub fn fesc(i: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(&[FESC])(i)
}

pub const TFEND: u8 = 0xDC;
pub fn tfend(i: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(&[TFEND])(i)
}

pub const TFESC: u8 = 0xDB;
pub fn tfesc(i: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(&[TFESC])(i)
}

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

pub fn frame_type_port(i: &[u8]) -> IResult<&[u8], (u8, FrameType)> {
    let (rest, byte_a) = take(1usize)(i)?;
    let byte = byte_a[0];
    let port = byte >> 4;
    // this just seems really ugly to me, I'm sure there's a better way to do it.
    // this validates that FrameType::Return can only be port 0x0F and also
    // that the frame type is a real frame type
    // also this is currently set up to use an unnecessary take(1usize) since we're
    // using a closure, so ... yea, this works, but needs some work
    // really, I think I just need to figure out how to return errors proper
    // because that's really all I'm doing with the map_opt
    // TODO yea, byte_a is totally useless here, but I don't know how to get rid of it.
    let (_, frame_type) = map_opt(be_u8, |_| {
        let frame_type = num::FromPrimitive::from_u8(byte & 0x0F)?;
        if frame_type == FrameType::Return && port != 0x0F {
            return None;
        }
        Some(frame_type)
    })(byte_a)?;

    Ok((rest, (port, frame_type)))
}

pub fn frame_content(i: Vec<u8>) -> IResult<&[u8], Frame> {
    let i = i.as_slice();
    let (rest, (port, frame_type)) = frame_type_port(i)?;

    let (rest, payload) = match frame_type {
        FrameType::Data => data_frame(rest),
        FrameType::TXDelay => txdelay_frame(rest),
        FrameType::P => p_frame(rest),
        FrameType::SlotTime => slot_time_frame(rest),
        FrameType::TXTail => txtail_frame(rest),
        FrameType::FullDuplex => fullduplex_frame(rest),
        FrameType::SetHardware => sethardware_frame(rest),
        FrameType::Return => Ok((rest, Payload::Return)),
    }?;

    // TODO asert that there's nothing left unparsed here

    Ok((
        rest,
        Frame {
            port: port,
            payload: payload,
        },
    ))
}

pub fn fesc_tfend(i: &[u8]) -> IResult<&[u8], u8> {
    let (rest, _) = tag(&[FESC, TFEND])(i)?;
    Ok((rest, FEND))
}

pub fn fesc_tfesc(i: &[u8]) -> IResult<&[u8], u8> {
    let (rest, _) = tag(&[FESC, TFESC])(i)?;
    Ok((rest, FESC))
}

// From the spec:
// Receipt of any character other than TFESC or TFEND while in escaped mode is an error; no action is taken and frame assembly continues.
// I'm choosing to interpret this as the FESC is discarded and the non TFESC/TFEND byte goes through without issue
pub fn fesc_other(i: &[u8]) -> IResult<&[u8], u8> {
    let (rest, _) = fesc(i)?;
    be_u8(rest)
}

pub fn take_frame_data(i: &[u8]) -> IResult<&[u8], u8> {
    alt((fesc_tfend, fesc_tfesc, fesc_other, be_u8))(i)
}

// change this to be a wrapper around some parser. so, similar to
// map_parser, have this map_unescaped_frame_data
// with_unescaped_frame_data
pub fn unescape_frame_data(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    fold_many0(take_frame_data, Vec::new(), |mut acc: Vec<_>, item| {
        acc.push(item);
        acc
    })(i)
}

pub fn parse_frame(i: &[u8]) -> IResult<&[u8], Frame> {
    // TODO take while not FEND
    //
    delimited(
        fend,
        map_parser(
            take_till(|b: u8| b == FEND),
            map_parser(unescape_frame_data, frame_content),
        ),
        fend,
    )(i)
}

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

    // TODO move the cases from `test_frame_type` into here
    #[test]
    fn test_frame_type_port() {
        let data = &[0x50 | FrameType::P as u8];
        assert_eq!(Ok((EMPTY, (5u8, FrameType::P))), frame_type_port(data));

        let data = &[0xF0 | FrameType::Return as u8];
        assert_eq!(
            Ok((EMPTY, (0x0F, FrameType::Return))),
            frame_type_port(data)
        );

        let data = &[0x0F];
        assert_ne!(
            Ok((EMPTY, (0x00, FrameType::Return))),
            frame_type_port(data)
        );
    }

    // #[test]
    // fn test_frame_type() {
    //     assert_eq!(
    //         Ok((EMPTY, FrameType::Data)),
    //         frame_type(&[FrameType::Data as u8])
    //     );
    //     assert_eq!(
    //         Ok((EMPTY, FrameType::TXDelay)),
    //         frame_type(&[FrameType::TXDelay as u8])
    //     );
    //     assert_eq!(Ok((EMPTY, FrameType::P)), frame_type(&[FrameType::P as u8]));
    //     assert_eq!(
    //         Ok((EMPTY, FrameType::SlotTime)),
    //         frame_type(&[FrameType::SlotTime as u8])
    //     );
    //     assert_eq!(
    //         Ok((EMPTY, FrameType::TXTail)),
    //         frame_type(&[FrameType::TXTail as u8])
    //     );
    //     assert_eq!(
    //         Ok((EMPTY, FrameType::FullDuplex)),
    //         frame_type(&[FrameType::FullDuplex as u8])
    //     );
    //     assert_eq!(
    //         Ok((EMPTY, FrameType::SetHardware)),
    //         frame_type(&[FrameType::SetHardware as u8])
    //     );
    //     assert_eq!(
    //         Ok((EMPTY, FrameType::Return)),
    //         frame_type(&[FrameType::Return as u8])
    //     );
    // }

    #[test]
    fn test_take_frame_data() {
        assert_eq!(Ok((EMPTY, 0x00)), take_frame_data(&[0x00]));

        let rest: &[u8] = &[0x42];
        let data: &[u8] = &[0x00, 0x42];
        assert_eq!(Ok((rest, 0x00)), take_frame_data(data));

        let data: &[u8] = &[FESC, TFEND];
        assert_eq!(Ok((EMPTY, FEND)), take_frame_data(data));

        let rest: &[u8] = &[0x42];
        let data: &[u8] = &[FESC, TFEND, 0x42];
        assert_eq!(Ok((rest, FEND)), take_frame_data(data));

        let data: &[u8] = &[FESC, TFESC];
        assert_eq!(Ok((EMPTY, FESC)), take_frame_data(data));

        let rest: &[u8] = &[0x42];
        let data: &[u8] = &[FESC, TFESC, 0x42];
        assert_eq!(Ok((rest, TFESC)), take_frame_data(data));
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
    fn test_frame_content() {
        let data = &[0x20 | FrameType::Data as u8, 42, 43];
        assert_eq!(
            Ok((
                EMPTY,
                Frame {
                    port: 2,
                    payload: Payload::Data(vec![42, 43])
                }
            )),
            frame_content(data)
        );

        let data = &[0x30 | FrameType::TXDelay as u8, 42];
        assert_eq!(
            Ok((
                EMPTY,
                Frame {
                    port: 3,
                    payload: Payload::TXDelay(42)
                }
            )),
            frame_content(data)
        );

        let data = &[0x00 | FrameType::P as u8, 42];
        assert_eq!(
            Ok((
                EMPTY,
                Frame {
                    port: 0,
                    payload: Payload::P(42)
                }
            )),
            frame_content(data)
        );

        let data = &[0xC0 | FrameType::SlotTime as u8, 42];
        assert_eq!(
            Ok((
                EMPTY,
                Frame {
                    port: 0x0C,
                    payload: Payload::SlotTime(42)
                }
            )),
            frame_content(data)
        );

        let data = &[0xF0 | FrameType::TXTail as u8, 42];
        assert_eq!(
            Ok((
                EMPTY,
                Frame {
                    port: 0x0F,
                    payload: Payload::TXTail(42)
                }
            )),
            frame_content(data)
        );

        let data = &[0x30 | FrameType::FullDuplex as u8, 0];
        assert_eq!(
            Ok((
                EMPTY,
                Frame {
                    port: 3,
                    payload: Payload::FullDuplex(false)
                }
            )),
            frame_content(data)
        );

        let data = &[0x70 | FrameType::FullDuplex as u8, 42];
        assert_eq!(
            Ok((
                EMPTY,
                Frame {
                    port: 7,
                    payload: Payload::FullDuplex(true)
                }
            )),
            frame_content(data)
        );

        let data = &[0x80 | FrameType::SetHardware as u8, 42, 43];
        assert_eq!(
            Ok((
                EMPTY,
                Frame {
                    port: 8,
                    payload: Payload::SetHardware(vec![42, 43])
                }
            )),
            frame_content(data)
        );

        let data = &[0x30 | FrameType::Return as u8];
        assert_ne!(
            Ok((
                EMPTY,
                Frame {
                    port: 0x0F,
                    payload: Payload::Return
                }
            )),
            frame_content(data)
        );

        let data = &[0xF0 | FrameType::Return as u8];
        assert_eq!(
            Ok((
                EMPTY,
                Frame {
                    port: 0x0F,
                    payload: Payload::Return
                }
            )),
            frame_content(data)
        );
    }

    #[test]
    fn send_the_characters_test_out_of_tnc_port_0() {
        let frame_data = &[0xC0, 0x00, 0x54, 0x45, 0x53, 0x54, 0xC0];
        assert_eq!(
            Ok((
                EMPTY,
                Frame {
                    port: 0,
                    payload: Payload::Data(vec![0x54, 0x45, 0x53, 0x54])
                }
            )),
            parse_frame(frame_data)
        );
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
