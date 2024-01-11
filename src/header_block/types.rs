#[derive(Debug)]
pub struct RcpFile {
    pub header: HeaderBlock,
}

#[derive(Debug)]
pub struct HeaderBlock {
    pub version: [u8; 32],
    pub title: [u8; 64],
    pub memo: [u8; 336],

    pub time_base: u8,
    pub tempo: u8,
    pub time_signature: TimeSignature,
    pub key: u8,
    pub play_bias: u8,
    pub cm6_file_name: [u8; 16],
    pub gsd_file_name: [u8; 16],
    pub number_of_tracks: u8,

    pub rhythm_notes: [RhythmNote; 32],
    pub user_exclusives: [UserExclusive; 8],
}

#[derive(Debug)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

#[derive(Debug)]
pub struct RhythmNote {
    pub name: [u8; 14],
    pub note_number: u8,
    pub gate_type: u8,
}

#[derive(Debug)]
pub struct UserExclusive {
    pub message: [u8; 48],
}

#[derive(Debug)]
pub struct TrackHeader {
    size: u16,
    channel: u8,
    key: u8,
    step: u8,
    mode: u8,
    comment: u8,
    data: Vec<u8>,
}

#[derive(Debug)]
pub struct TrackEvent {
    event_type: u8,
    step_time: u8,
    gate_time: u8,
    velocity: u8,
}
