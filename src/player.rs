use anyhow::Context;
use rodio::Source;
use std::io::Cursor;

static NOISE: &[u8] = include_bytes!("../assets/mynoise_gray_05.mp3");

pub fn loop_forever() -> anyhow::Result<()> {
    let (_stream, handle) =
        rodio::OutputStream::try_default().context("can't open audio device")?;
    let sink = rodio::Sink::try_new(&handle).context("can't create audio sink")?;
    sink.set_volume(0.3);

    let source = rodio::Decoder::new(Cursor::new(NOISE)).context("can't decode")?.repeat_infinite();
    sink.append(source);

    // Since the source never ends, this is equivalent to sleeping forever.
    sink.sleep_until_end();

    Ok(())
}
