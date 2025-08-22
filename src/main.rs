use std::{
    env,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

mod memory_mapping;

use walkdir::WalkDir;

use crate::memory_mapping::{MemoryMap, MemoryMapping};

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

#[derive(Debug)]
enum GamePatchOffsetType {
    Raw,
    Virtual,
}

#[derive(Debug)]
struct GamePatch {
    name: String,
    offset: u32,
    offset_type: GamePatchOffsetType,
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
    let xbe_path = match find_file_in_folder("default.xbe".into(), PathBuf::from(&extraction_dir)) {
        Some(x) => x,
        None => error_exit("Unable to find default.xbe in the .iso files."),
    };

    match backup_file(&xbe_path) {
        Ok(b) => b,
        Err(_) => {
            error_exit("Unable to blah blah blah");
        }
    };

    // Parse the config

    // Write the patches
    // TODO: Generate the memory map from the .xbe file instead of having to manually enter it
    let mem = MemoryMap::new(vec![
        MemoryMapping {
            file_start: 0x0,
            virtual_start: 0x00010000,
            size: 0xf60,
        },
        MemoryMapping {
            file_start: 0x1000,
            virtual_start: 0x00011000,
            size: 0x160020,
        },
    ]);

    let mut xbe_writer =
        XBEWriter::new(&xbe_path, mem).expect("Unable to create new XBE writer to apply patches.");

    let patches = vec![GamePatch {
        name: String::from("Force 1sec cutscenes"),
        offset: 0x4d5ac,
        offset_type: GamePatchOffsetType::Virtual,
        replacement_bytes: vec![0x90, 0x90, 0x90, 0x90, 0x90, 0x90],
        original_bytes: Some(vec![0xf3, 0x0f, 0x10, 0x3c, 0x24, 0x08]),
    }];

    patches.iter().for_each(|p| {
        // TODO: Remove these unwraps
        print!("Applying patch \"{}\"...  ", &p.name);
        std::io::stdout().flush().expect("Unable to flush stdout");

        match xbe_writer.apply_patch(p) {
            Ok(_) => println!("DONE!"),
            Err(_) => println!("FAILED!\n        Failed to apply {}.", &p.name),
        };
    });

    println!("All patches applied successfully.");
}

#[derive(Debug)]
struct XBEWriter {
    xbe_file: File,
    mem_map: MemoryMap,
}

impl XBEWriter {
    pub fn new(path: &PathBuf, mem_map: MemoryMap) -> Result<XBEWriter, std::io::Error> {
        let xbe = match OpenOptions::new().read(true).write(true).open(&path) {
            Ok(f) => f,
            Err(_) => error_exit(format!(
                "Unable to open file {} for writing.",
                path.to_str().unwrap()
            )),
        };

        Ok(XBEWriter {
            xbe_file: xbe,
            mem_map,
        })
    }

    pub fn apply_patch(&mut self, patch: &GamePatch) -> Result<(), std::io::Error> {
        let offset: u64 = match patch.offset_type {
            GamePatchOffsetType::Raw => patch.offset.into(),
            GamePatchOffsetType::Virtual => self.mem_map.get_raw_offset(patch.offset)?.into(),
        };

        self.xbe_file.seek(SeekFrom::Start(offset));
        self.xbe_file.write(patch.replacement_bytes.as_ref())?;
        Ok(())
    }
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

// #[must_use]
fn backup_file(filepath: &PathBuf) -> Result<PathBuf, std::io::Error> {
    let mut new_filepath: PathBuf = filepath.clone();

    if let Some(filename) = filepath.file_name() {
        let mut backup_name = filename.to_os_string();
        backup_name.push(".bak");
        new_filepath.set_file_name(backup_name);

        // Do not overwrite if it exists
        // TODO: Make this an option or prompt to the user
        if new_filepath.exists() {
            return Ok(new_filepath);
        }

        std::fs::copy(filepath, &new_filepath)?;
        Ok(new_filepath)
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "File has no filename",
        ));
    }
}

fn execute_command(command: &str, args: std::process::CommandArgs) -> Result<(), std::io::Error> {
    // If the folder doesn't exist (or the user just deleted it), then extract the game
    if cfg!(target_os = "windows") {
        todo!();
    } else {
        let mut extractor = Command::new(command)
            .args(args)
            .stdout(Stdio::piped())
            .spawn()?;
        // .expect("Failed to start extract-xiso");

        let estd = extractor.stdout.take().ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to capture stdout",
        ))?;

        let command_stdout_reader = BufReader::new(estd);

        let reader = std::thread::spawn(move || {
            for line in command_stdout_reader.lines() {
                if let Ok(line) = line {
                    print!("{}", line);
                    std::thread::sleep(Duration::from_millis(10));
                };
            }
        });
        extractor.wait()?;
        // .expect("Failed to wait on extract-xiso");
        reader.join();
        // .expect("Unable to join reader thread.");
    };

    Ok(())
}
