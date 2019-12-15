use std::io;

use main_error::MainError;

mod srt;

fn main() -> Result<(), MainError> {
    let stdin = io::stdin();
    let subs = srt::from_reader(stdin)?;

    for sub in subs {
        println!("{:?}", sub);
    }

    Ok(())
}
