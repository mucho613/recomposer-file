use std::vec;

use nom::{
    bytes::complete::take,
    error::Error,
    number::complete::{be_u16, be_u8},
    IResult,
};

use crate::event::TrackEvent;

use super::types::{Track, TrackBlock, TrackHeader, TrackType};

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
        0x00..=0x7F => TrackEvent::Note {
            step_time: byte_0,
            key_number: event_type,
            gate_time: byte_1,
            velocity: byte_2,
        },

        0x90..=0x97 => TrackEvent::UserExclusive {
            step_time: byte_0,
            number: event_type & 0x0F,
        },

        0xDD => TrackEvent::RolandBaseAddress {
            step_time: byte_0,
            gate_time: byte_1,
            velocity: byte_2,
        },
        0xDE => TrackEvent::RolandParameter {
            step_time: byte_0,
            gate_time: byte_1,
            velocity: byte_2,
        },
        0xDF => TrackEvent::RolandDeviceNumber {
            step_time: byte_0,
            gate_time: byte_1,
            velocity: byte_2,
        },

        0xE2 => TrackEvent::BankPrg {
            step_time: byte_0,
            gate_time: byte_1,
            velocity: byte_2,
        },
        0xE5 => TrackEvent::Keyin {
            step_time: byte_0,
            gate_time: byte_1,
            velocity: byte_2,
        },
        0xE6 => TrackEvent::MidiChannel {
            step_time: byte_0,
            channel: byte_1,
        },
        0xE7 => TrackEvent::Tempo {
            step_time: byte_0,
            tempo: (byte_2 as u16) << 7 | byte_1 as u16,
        },
        0xEA => TrackEvent::AfterTouch {
            step_time: byte_0,
            pressure: byte_1,
        },
        0xEB => TrackEvent::ControlChange {
            step_time: byte_0,
            controller_number: byte_1,
            value: byte_2,
        },
        0xEC => TrackEvent::ProgramChange {
            step_time: byte_0,
            program_number: byte_1,
        },
        0xED => TrackEvent::PolyphonicAfterTouch {
            step_time: byte_0,
            key_number: byte_1,
            pressure: byte_2,
        },
        0xEE => TrackEvent::PitchBend {
            step_time: byte_0,
            value: (byte_2 as i16) << 7 | byte_1 as i16,
        },

        0xF5 => TrackEvent::Key { offset: byte_0 },
        0xF8 => TrackEvent::RepeatEnd { count: byte_0 },
        0xF9 => TrackEvent::RepeatStart,
        0xFC => TrackEvent::SameMeasure {
            measure: byte_0,
            track_offset: (byte_2 as u16) << 7 | byte_1 as u16,
        },
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

    Ok((
        i2,
        TrackEvent::TrackExclusive {
            step_time,
            message_body: buffer,
        },
    ))
}

fn take_comment_event(i: &[u8]) -> IResult<&[u8], TrackEvent, Error<&[u8]>> {
    let (_, event_type) = be_u8(i)?;
    if event_type != 0xF6 {
        panic!("Not comment event");
    }

    let (i, bytes) = take(40usize)(i)?;

    // 40 bytes のうち、4 bytes ごとの 3, 4 byte 目だけを取り出して、Array に詰める
    let buffer = bytes
        .chunks(4)
        .map(|chunk| chunk[2..4].try_into().unwrap())
        .flatten()
        .collect::<Vec<u8>>();

    let mut text = [0u8; 20];
    text.copy_from_slice(&buffer[0..20]);

    Ok((i, TrackEvent::Comment { text }))
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
