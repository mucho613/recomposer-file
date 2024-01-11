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

#[derive(Debug)]
enum TrackEvent {
    Note(StepTime, GateTime, Velocity),
    UserExclusive(StepTime),
    TrackExclusive(Vec<u8>),
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
    PitchBend(i16),
    Key(u8),
    Comment(Vec<u8>),
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
