use serde_json::Value;
use std::{
    io::{self, BufWriter, Write},
    sync::mpsc::{channel, Receiver, Sender},
};
use tokio::task::{spawn_blocking, JoinHandle};

pub fn start(writer: impl Write + Send + 'static) -> (JoinHandle<()>, Sender<Value>) {
    let (write_tx, write_rx) = channel();
    let handle = spawn_blocking(move || write_messages(writer, write_rx));
    (handle, write_tx)
}

fn write_messages(writer: impl Write, write_rx: Receiver<Value>) {
    let mut writer = BufWriter::new(writer);

    while let Ok(value) = write_rx.recv() {
        if let Err(err) = write_value(&mut writer, value) {
            eprintln!("failed to write response to output: {}", err);
            break;
        }
    }
}

fn write_value(mut writer: &mut impl Write, value: Value) -> io::Result<()> {
    serde_json::to_writer(&mut writer, &value)?;
    writeln!(writer)?;
    writer.flush()
}
