use std::{
    collections::HashMap,
    io::{stdin, stdout, Read, Stdin, Stdout, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "keebox",
    author = "Joris Klein Tijssink",
    about = "A minimal CLI utility to manage a multi-tentant keybox.",
    rename_all = "kebab"
)]
struct GlobalArgs {
    #[structopt(about = "the keebox file", parse(from_os_str))]
    file: PathBuf,

    #[structopt(about = "the name of the key to get")]
    key: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = GlobalArgs::from_args();

    // Read the file.
    let content = std::fs::read_to_string(&args.file)
        .with_context(|| format!("could not open file: {:?}", &args.file))?;

    // Parse the file.
    let mut map: HashMap<String, String> =
        serde_json::from_str(&content).with_context(|| format!("json contents are malformed"))?;

    // Prepare stdin/stdout.
    let mut stdout: Stdout = stdout();

    // Grab the current value.
    // let key = args.name.clone();
    let current_value = match map.get(&args.key) {
        Some(v) => v.clone(),
        None => String::from("other"),
    };

    // Output the current value.
    stdout.write(current_value.as_bytes())?;

    // Test if stdin is a tty. If it is, it *isn't* a pipe.
    let stdin: Stdin = stdin();
    if atty::is(atty::Stream::Stdin) {
        return Ok(());
    }

    // Read stdin until the end.
    let mut input: Vec<u8> = Vec::new();
    stdin.lock().read_to_end(&mut input)?;

    if input.len() == 0 {
        // If stdin was a pipe, but no data was passed, delete the key.
        map.remove(&args.key);
        return Ok(());
    } else {
        // Otherwise, store the new value.
        map.insert(
            args.key,
            String::from_utf8(input)
                .map(|str| str.trim().to_string())
                .with_context(|| format!("input data is not valid utf-8"))?,
        );
    }

    // Reserialize the map to the file.
    std::fs::write(
        &args.file,
        serde_json::to_string_pretty(&map)
            .with_context(|| format!("failed to serialize new json value"))?,
    )
    .with_context(|| format!("failed to write to file: {:?}", &args.file))?;

    Ok(())
}
