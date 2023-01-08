use std::convert::TryFrom;

pub fn handle_midi_message<T>(
    _stamp: u64,
    bytes: &[u8],
    _trash: &mut T,
) -> Result<f32, wmidi::FromBytesError> {
    let message = wmidi::MidiMessage::try_from(bytes)?;
    if let wmidi::MidiMessage::NoteOn(_, note, _) = message {
        return Ok(play_note(note));
    }
    Ok(0f32)
}

pub fn play_note(note: wmidi::Note) -> f32 {
    note.to_freq_f32()
}

pub fn play(sample: f64) -> () {}
