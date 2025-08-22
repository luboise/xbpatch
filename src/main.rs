use std::{
    env,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use walkdir::WalkDir;

#[derive(Debug, Default)]
struct XBPatchArgs {
    iso_path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    unexpected_args: Vec<String>,
}

#[derive(PartialEq)]
enum ArgParseState {
    NoState,
    ExpectingFilepath,
    ExpectingConfigpath,
    UnexpectedArg,
}

#[derive(Debug)]
enum ArgError {
    InvalidArgState,
    IsoSpecifiedMultipleTimes,
}

impl MemoryMapping {
    fn contains_block(&self, block_start: usize, block_size: usize) -> bool {
        block_start < self.size && (self.size - block_start - block_size) < self.size
    }
}

#[derive(Debug)]
struct MemoryMapping {
    live_offset: u32,
    size: usize,
    file_offset: u32,
}

#[derive(Debug)]
struct GamePatch {
    name: String,
    patch_offset: u32,
    replacement_bytes: Vec<u8>,
    original_bytes: Option<Vec<u8>>,
}

#[derive(Debug)]
struct XBPatchConfig {
    mem_mappings: Vec<MemoryMapping>,
    patches: Vec<GamePatch>,
}

// Usage
// xbpatch gbtg.iso --config gbtg.xbconf

fn main() {
    // Parse args
    let mut args = match parse_args(env::args()) {
        Ok(a) => a,
        Err(e) => match e {
            ArgError::InvalidArgState => {
                error_exit("Unable to proceed: Invalid arguments.");
            }
            ArgError::IsoSpecifiedMultipleTimes => {
                error_exit("Unable to proceed: Iso has been specified multiple times.");
            }
        },
    };

    // Extract the ISO
    let iso: PathBuf = args.iso_path.take().expect("Expected an iso path.");

    if !iso.exists() {
        error_exit("The iso provided does not exist.");
    } else if !iso.is_file() {
        error_exit("The iso provided is not a file.");
    }

    let extraction_dir: PathBuf = "./xbpatch_temp/isos/isoname".into();

    // If the folder exists and the user wants to delete it, then delete it
    if extraction_dir.exists() && extraction_dir.is_dir() {
        let skip_extraction = prompt_user_bool(format!(
            "Extraction dir {} already exists.\nWould you like to skip extraction and use this folder instead?",
            extraction_dir.to_str().unwrap()
        ));
        if !skip_extraction {
            std::fs::remove_dir_all(&extraction_dir).expect("Unable to delete existing folder.");
        }
    }

    // If the folder doesn't exist (or the user just deleted it), then extract the game
    if !extraction_dir.exists() {
        std::fs::create_dir_all(&extraction_dir).expect("Unable to create xbpatch dir.");

        if cfg!(target_os = "windows") {
            todo!();
        } else {
            let mut extractor = Command::new("extract-xiso")
                .arg("-x")
                .arg(&iso)
                .arg("-d")
                .arg(&extraction_dir)
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to start extract-xiso");

            let xiso_stdout_reader =
                BufReader::new(extractor.stdout.take().expect("Failed to capture stdout"));

            let reader = std::thread::spawn(move || {
                for line in xiso_stdout_reader.lines() {
                    if let Ok(line) = line {
                        print!("{}", line);
                        std::thread::sleep(Duration::from_millis(10));
                    };
                }
            });
            extractor.wait().expect("Failed to wait on extract-xiso");
            reader.join().expect("Unable to join reader thread.");
        };
    }

    // Grab default.xbe out of it
    let xbe = match find_file_in_folder("default.xbe".into(), PathBuf::from(&extraction_dir)) {
        Some(x) => x,
        None => error_exit("Unable to find default.xbe in the .iso files."),
    }

    // Parse the config
}

fn prompt_user_bool(msg: String) -> bool {
    print!("{} (y/n): ", msg);
    std::io::stdout().flush().expect("Unable to flush stdout.");

    let mut user_input = String::new();
    std::io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    user_input.to_lowercase().starts_with("y")
}

fn parse_args(args: env::Args) -> Result<XBPatchArgs, ArgError> {
    let mut state: ArgParseState = ArgParseState::NoState;
    let mut ret_args: XBPatchArgs = Default::default();

    for arg in args.skip(1) {
        match state {
            ArgParseState::NoState => {
                if arg.starts_with('-') {
                    if arg.starts_with("--") {
                        if arg == "--config" {
                            state = ArgParseState::ExpectingConfigpath;
                        } else {
                            state = ArgParseState::UnexpectedArg;
                            ret_args.unexpected_args.push(arg);
                        }
                    }
                } else {
                    if ret_args.iso_path.is_some() {
                        return Err(ArgError::IsoSpecifiedMultipleTimes);
                    }

                    ret_args.iso_path = Some(arg.into());
                }
            }
            ArgParseState::ExpectingFilepath => {
                ret_args.iso_path = Some(arg.into());
                state = ArgParseState::NoState
            }
            ArgParseState::ExpectingConfigpath => {
                ret_args.config_path = Some(arg.into());
                state = ArgParseState::NoState
            }
            ArgParseState::UnexpectedArg => state = ArgParseState::NoState,
        }
    }

    if state == ArgParseState::NoState {
        Ok(ret_args)
    } else {
        Err(ArgError::InvalidArgState)
    }
}

fn error_exit_with_details(message: impl AsRef<str>, details: impl AsRef<str>) -> ! {
    eprintln!("Unable to continue.");
    eprint!("Error: {}\nDetails: {}", message.as_ref(), details.as_ref());
    std::process::exit(1);
}

fn error_exit(message: impl AsRef<str>) -> ! {
    eprintln!("Unable to continue.");
    eprint!("Error: {}", message.as_ref());
    std::process::exit(1);
}

fn find_file_in_folder(file: PathBuf, folder: PathBuf) -> Option<PathBuf> {
    for entry in WalkDir::new(folder) {
        if let Ok(entry) = entry {
            if entry.path().file_name() == file.file_name() {
                return Some(entry.path().into());
            }
        }
    }

    None
}
