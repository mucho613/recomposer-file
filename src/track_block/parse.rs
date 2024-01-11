use nom::{
    bytes::complete::take,
    error::Error,
    number::complete::{be_u16, be_u8},
    IResult,
};

use super::types::{Track, TrackBlock, TrackEvent, TrackHeader, TrackType};

fn parse_track_header(i: &[u8]) -> IResult<&[u8], TrackHeader, Error<&[u8]>> {
    let (i, size) = be_u16(i)?;
    let (i, track_number) = be_u8(i)?;
    let (i, track_type) = be_u8(i)?;
    let (i, channel) = be_u8(i)?;
    let (i, key_bias) = be_u8(i)?;
    let (i, step_bias) = be_u8(i)?;
    let (i, is_muting) = be_u8(i)?;
    let (i, comment) = take(36usize)(i)?;

    let track_type = match track_type {
        0x00 => TrackType::Normal,
        0x80 => TrackType::Rhythm,
        _ => panic!("Unknown track type"),
    };

    let is_muting = match is_muting {
        0x00 => false,
        0x01 => true,
        _ => panic!("Unknown value for is_muting"),
    };

    Ok((
        i,
        TrackHeader {
            size,
            track_number,
            track_type,
            channel,
            key_bias,
            step_bias,
            muting: is_muting,
            comment: comment.try_into().unwrap(),
        },
    ))
}

pub fn parse_track_event(i: &[u8]) -> IResult<&[u8], TrackEvent, Error<&[u8]>> {
    let (i, event_type) = be_u8(i)?;
    let (i, byte_0) = be_u8(i)?;
    let (i, byte_1) = be_u8(i)?;
    let (i, byte_2) = be_u8(i)?;

    let track_event = match event_type {
        0x00..=0x7F => TrackEvent::Note(byte_0, byte_1, byte_2),

        0x90 => TrackEvent::UserExclusive(byte_0),
        0x98 => TrackEvent::TrackExclusive(byte_0, byte_1),

        0xDD => TrackEvent::RolBase(byte_0, byte_1, byte_2),
        0xDE => TrackEvent::RolPara(byte_0, byte_1, byte_2),
        0xDF => TrackEvent::RolDev(byte_0, byte_1, byte_2),

        0xE2 => TrackEvent::BankPrg(byte_0, byte_1, byte_2),
        0xE5 => TrackEvent::Keyin(byte_0, byte_1, byte_2),
        0xE6 => TrackEvent::MidiChannel(byte_0, byte_1),
        0xE7 => TrackEvent::Tempo(byte_0, byte_1, byte_2),
        0xEA => TrackEvent::AfterTouch(byte_0, byte_1),
        0xEB => TrackEvent::Control(byte_0, byte_1, byte_2),
        0xEC => TrackEvent::ProgramChange(byte_0, byte_1),
        0xED => TrackEvent::PolyphonicAfterTouch(byte_0, byte_1, byte_2),
        0xEE => TrackEvent::PitchBend((byte_1 as i16) << 7 | byte_0 as i16),

        0xF5 => TrackEvent::Key(byte_0),
        0xF6 => TrackEvent::Comment(byte_0, byte_1),
        0xF7 => TrackEvent::ContinuesData(byte_0, byte_1),
        0xF8 => TrackEvent::RepeatEnd(byte_0),
        0xF9 => TrackEvent::RepeatStart,
        0xFC => TrackEvent::SameMeasure,
        0xFD => TrackEvent::BarLine,
        0xFE => TrackEvent::EndOfTrack,

        _ => panic!("Unknown track event type"),
    };

    Ok((i, track_event))
}

fn parse_track(i: &[u8]) -> IResult<&[u8], Track, Error<&[u8]>> {
    let (mut i, track_header) = parse_track_header(i)?;

    let mut track_events = vec![];

    loop {
        let (i2, event) = parse_track_event(i)?;
        i = i2;

        if let TrackEvent::EndOfTrack = event {
            break;
        }

        track_events.push(event);
    }

    Ok((
        i,
        Track {
            track_header,
            track_events,
        },
    ))
}

pub fn parse_track_block(i: &[u8]) -> IResult<&[u8], TrackBlock, Error<&[u8]>> {
    let mut i = i;

    let mut tracks = vec![];

    (0..18).for_each(|_| {
        let (i2, track) = parse_track(i).unwrap();
        i = i2;
        tracks.push(track);
    });

    Ok((i, TrackBlock { tracks }))
}
