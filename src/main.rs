use std::io;

use main_error::MainError;

mod srt;

fn main() -> Result<(), MainError> {
    let subs = srt::from_reader(io::stdin())?;

    let mut wtr = csv::Writer::from_writer(io::stdout());

    wtr.write_record(&["index", "start", "end", "text"])?;
    for srt::SubTitle { index, start, end, text } in subs {
        let index = index.to_string();
        let start = start.to_duration().as_millis().to_string();
        let end = end.to_duration().as_millis().to_string();
        wtr.write_record(&[&index, &start, &end, &text])?;
    }

    wtr.flush()?;

    Ok(())
}
