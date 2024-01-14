#[derive(Debug)]
pub struct TrackBlock {
    pub tracks: Vec<Track>,
}

#[derive(Debug)]
pub struct Track {
    pub track_header: TrackHeader,
    pub track_events: Vec<TrackEvent>,
}

#[derive(Debug)]
pub enum TrackType {
    Normal,
    Rhythm,
}

#[derive(Debug)]
pub struct TrackHeader {
    pub size: u16,
    pub track_number: u8,
    pub track_type: TrackType,
    pub channel: u8,
    pub key_bias: u8,
    pub step_bias: u8,
    pub muting: bool,
    pub comment: [u8; 36],
}

#[derive(Debug)]
pub enum TrackEvent {
    Note(KeyNumber, StepTime, GateTime, Velocity),
    UserExclusive(StepTime, u8),
    TrackExclusive(StepTime, Vec<u8>),
    TrackExclusiveStart(StepTime, u8, u8),
    RolBase(StepTime, GateTime, Velocity),
    RolPara(StepTime, GateTime, Velocity),
    RolDev(StepTime, GateTime, Velocity),
    BankPrg(StepTime, GateTime, Velocity),
    Keyin(StepTime, GateTime, Velocity),
    MidiChannel(StepTime, GateTime),
    Tempo(StepTime, GateTime, Velocity),
    AfterTouch(StepTime, Pressure),
    Control(StepTime, u8, u8),
    ProgramChange(StepTime, u8),
    PolyphonicAfterTouch(StepTime, KeyNumber, Pressure),
    PitchBend(StepTime, i16),
    Key(u8),
    Comment(Vec<u8>),
    CommentStart(u8, u8),
    ContinuesData(u8, u8),
    RepeatEnd(RepeatCount),
    RepeatStart,
    SameMeasure,
    BarLine,
    EndOfTrack,
}

type StepTime = u8;
type GateTime = u8;
type Velocity = u8;
type KeyNumber = u8;
type Pressure = u8;

type RepeatCount = u8;
