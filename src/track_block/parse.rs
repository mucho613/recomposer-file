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

pub fn parse_track_event(i: &[u8]) -> IResult<&[u8], TrackEvent, Error<&[u8]>> {
    let (i, event_type) = be_u8(i)?;
    let (i, byte_0) = be_u8(i)?;
    let (i, byte_1) = be_u8(i)?;
    let (i, byte_2) = be_u8(i)?;

    let track_event = match event_type {
        0x00..=0x7F => TrackEvent::Note(event_type, byte_0, byte_1, byte_2),

        0x90 => TrackEvent::UserExclusive(byte_0),
        0x98 => TrackEvent::TrackExclusiveStart(byte_1, byte_2),

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
        0xF6 => TrackEvent::CommentStart(byte_1, byte_2),
        0xF7 => TrackEvent::ContinuesData(byte_1, byte_2),
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

    enum BufferType {
        Comment,
        TrackExclusive,
    }
    let mut buffer_type: Option<BufferType> = None;
    let mut counter: usize = 0;
    let mut buffer: Vec<u8> = vec![];

    // CommentStart または TrackExclusiveStart が見つかったら、バッファにそれ以降の ContinuesData を貯めて、
    // 1つの Comment または TrackExclusive にマージする
    track_events =
        track_events
            .into_iter()
            .fold(vec![], |mut acc, track_event| match track_event {
                TrackEvent::CommentStart(byte_0, byte_1) => {
                    buffer_type = Some(BufferType::Comment);
                    buffer.clear();

                    buffer.push(byte_0);
                    buffer.push(byte_1);
                    acc
                }
                TrackEvent::TrackExclusiveStart(byte_0, byte_1) => {
                    buffer_type = Some(BufferType::TrackExclusive);
                    buffer.clear();

                    buffer.push(byte_0);
                    buffer.push(byte_1);
                    acc
                }
                TrackEvent::ContinuesData(byte_0, byte_1) => {
                    buffer.push(byte_0);
                    buffer.push(byte_1);
                    acc
                }
                _ => {
                    match buffer_type {
                        Some(BufferType::Comment) => {
                            acc.push(TrackEvent::Comment(buffer.clone()));
                        }
                        Some(BufferType::TrackExclusive) => {
                            // 末尾に複数個ある 0xF7 を取り除く
                            // TODO: 末尾に 0xF8 がある場合もあるらしいので対応する
                            let position = buffer.iter().rposition(|&x| x != 0xF7);
                            let position = match position {
                                Some(position) => position,
                                None => return acc,
                            };
                            buffer.truncate(position + 1);

                            acc.push(TrackEvent::TrackExclusive(buffer.clone()));
                        }
                        None => {
                            acc.push(track_event);
                        }
                    }
                    buffer_type = None;
                    acc
                }
            });

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
