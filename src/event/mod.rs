#[derive(Debug)]
pub enum TrackEvent {
    Note {
        step_time: u8,
        key_number: u8,
        gate_time: u8,
        velocity: u8,
    },
    UserExclusive {
        step_time: u8,
        number: u8,
    },
    TrackExclusive {
        step_time: u8,
        message_body: Vec<u8>,
    },
    RolandBaseAddress {
        step_time: u8,
        gate_time: u8,
        velocity: u8,
    },
    RolandDeviceNumber {
        step_time: u8,
        gate_time: u8,
        velocity: u8,
    },
    RolandParameter {
        step_time: u8,
        gate_time: u8,
        velocity: u8,
    },
    BankPrg {
        step_time: u8,
        gate_time: u8,
        velocity: u8,
    },
    Keyin {
        step_time: u8,
        gate_time: u8,
        velocity: u8,
    },
    MidiChannel {
        step_time: u8,
        channel: u8,
    },
    Tempo {
        step_time: u8,
        tempo: u16,
    },
    AfterTouch {
        step_time: u8,
        pressure: u8,
    },
    ControlChange {
        step_time: u8,
        controller_number: u8,
        value: u8,
    },
    ProgramChange {
        step_time: u8,
        program_number: u8,
    },
    PolyphonicAfterTouch {
        step_time: u8,
        key_number: u8,
        pressure: u8,
    },
    PitchBend {
        step_time: u8,
        value: i16,
    },
    Key {
        offset: u8,
    },
    Comment {
        text: [u8; 20],
    },
    RepeatEnd {
        count: u8,
    },
    RepeatStart,
    SameMeasure {
        measure: u8,
        track_offset: u16,
    },
    BarLine,
    EndOfTrack,
}
