use std::vec;

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

fn take_single_length_event(i: &[u8]) -> IResult<&[u8], TrackEvent, Error<&[u8]>> {
    let (i, event_type) = be_u8(i)?;
    let (i, byte_0) = be_u8(i)?;
    let (i, byte_1) = be_u8(i)?;
    let (i, byte_2) = be_u8(i)?;

    let track_event = match event_type {
        0x00..=0x7F => TrackEvent::Note(event_type, byte_0, byte_1, byte_2),

        0x90..=0x97 => TrackEvent::UserExclusive(byte_0, event_type & 0x0F),

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
        0xEE => TrackEvent::PitchBend(byte_0, (byte_2 as i16) << 7 | byte_1 as i16),

        0xF5 => TrackEvent::Key(byte_0),
        0xF8 => TrackEvent::RepeatEnd(byte_0),
        0xF9 => TrackEvent::RepeatStart,
        0xFC => TrackEvent::SameMeasure,
        0xFD => TrackEvent::BarLine,
        0xFE => TrackEvent::EndOfTrack,

        _ => panic!("Unknown track event type"),
    };

    Ok((i, track_event))
}

fn take_track_exclusive_event(i: &[u8]) -> IResult<&[u8], TrackEvent, Error<&[u8]>> {
    let (i, event_type) = be_u8(i)?;
    let (i, step_time) = be_u8(i)?;
    let (i, byte_2) = be_u8(i)?;
    let (i, byte_3) = be_u8(i)?;
    if event_type != 0x98 {
        panic!("Not track exclusive event");
    }

    let mut buffer = vec![];

    buffer.push(byte_2);
    buffer.push(byte_3);

    let mut i2 = i;

    loop {
        let (i, _) = be_u8(i2)?;
        let (i, _) = be_u8(i)?;
        let (i, byte_2) = be_u8(i)?;
        let (i, byte_3) = be_u8(i)?;

        i2 = i;

        buffer.push(byte_2);
        buffer.push(byte_3);

        if byte_3 == 0xF7 {
            break;
        }
    }

    Ok((i2, TrackEvent::TrackExclusive(step_time, buffer)))
}

fn take_comment_event(i: &[u8]) -> IResult<&[u8], TrackEvent, Error<&[u8]>> {
    let (i, event_type) = be_u8(i)?;
    let (i, _) = be_u8(i)?;
    let (i, byte_2) = be_u8(i)?;
    let (i, byte_3) = be_u8(i)?;
    if event_type != 0xF6 {
        panic!("Not comment event");
    }

    let mut buffer = vec![];

    buffer.push(byte_2);
    buffer.push(byte_3);

    let mut i2 = i;

    loop {
        let (i, _) = be_u8(i2)?;
        let (i, _) = be_u8(i)?;
        let (i, byte_2) = be_u8(i)?;
        let (i, byte_3) = be_u8(i)?;

        i2 = i;

        buffer.push(byte_2);
        buffer.push(byte_3);

        if buffer.len() >= 20 {
            break;
        }
    }

    Ok((i2, TrackEvent::Comment(buffer)))
}

pub fn parse_track_event(i: &[u8]) -> IResult<&[u8], TrackEvent, Error<&[u8]>> {
    let (_, event_type) = be_u8(i)?;

    let (i, track_event) = match event_type {
        0x98 => take_track_exclusive_event(i)?,
        0xF6 => take_comment_event(i)?,
        // イベントの区切りで 0xF7 が見つかるのは不正
        0xF7 => panic!("Invalid track event"),
        _ => take_single_length_event(i)?,
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
