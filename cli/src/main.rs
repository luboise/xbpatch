use std::{env, fs, io::Write, path::PathBuf};

use xbpatch_core::{
    iso_handling::{self, backup_file, restore_backup},
    patching::{Patch, PatchEntry, PatchOffsetType},
    xbe::{PatchReport, XBEWriter},
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

    // TODO: Make this a program argument
    let extract_xiso_path = PathBuf::from("extract-xiso");

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

        iso_handling::extract_iso(&extract_xiso_path, &iso, &extraction_dir)
            .unwrap_or_else(|_| eprintln!("Failed to extract iso."));
    }

    // Grab default.xbe out of it
    let xbe_path = match find_file_in_folder("default.xbe".into(), PathBuf::from(&extraction_dir)) {
        Some(x) => x,
        None => error_exit("Unable to find default.xbe in the .iso files."),
    };

    if restore_backup(&xbe_path)
        .expect("Failed to restore backup.")
        .is_none()
    {
        match backup_file(&xbe_path) {
            Ok(b) => b,
            Err(_) => {
                error_exit("Unable to backup default.xbe");
            }
        };
    }

    // Parse the config
    let mut xbe_writer = match XBEWriter::new(&xbe_path) {
        Ok(w) => w,
        Err(_) => error_exit(format!(
            "Unable to open file {} for writing.",
            &xbe_path.to_str().unwrap()
        )),
    };

    let mut patch_entries = Vec::new();
    patch_entries.push(PatchEntry::new(
        String::from("Uncap frame rate"),
        String::from("Uncaps the frame rate"),
        None,
        vec![
            Patch {
                offset: 0x154919,
                offset_type: PatchOffsetType::Virtual,
                replacement_bytes: vec![0xeb, 0x21],
                original_bytes: None,
            },
            Patch {
                offset: 0x81454,
                offset_type: PatchOffsetType::Virtual,
                replacement_bytes: vec![0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90],
                original_bytes: None,
            },
        ],
    ));

    let mut report = PatchReport::default();

    for entry in patch_entries {
        print!("Applying patch \"{}\"...  ", entry.name());

        match xbe_writer.apply_patches(&entry) {
            Ok(patch_report) => {
                if patch_report.patch_successful() {
                    report.add_success();

                    println!("DONE!");
                } else {
                    report.add_failure();
                    println!("FAILED!\n        Failed to apply {}.", entry.name());
                }
            }
            Err(e) => {
                error_exit("CRITICAL FAILURE!");
            }
        }
    }

    if report.failures() == 0 {
        println!("All patches applied successfully.");
    } else {
        if report.successes() == 0 {
            println!("Failed to apply all patches.");
            println!("Exiting now.");
            std::process::exit(0);
        }

        let should_continue = prompt_user_bool(format!(
            "Failed to apply {} patches. Would you like to write the iso anyway?",
            report.failures()
        ));

        if !should_continue {
            println!("Exiting now.");
            std::process::exit(0);
        }
    }

    if let Some(parent_dir) = extraction_dir.parent() {
        let iso_path = parent_dir.join("isoname.iso");
        println!(
            "Writing new iso at {}",
            &iso_path.as_os_str().to_str().unwrap()
        );

        match iso_handling::create_iso(&extract_xiso_path, &iso_path, &extraction_dir) {
            Ok(_) => (),
            Err(e) => error_exit_with_details("Unable to create iso", e.to_string()),
        };
        println!(
            "\nSuccessfully wrote new iso to {}.",
            &iso_path.to_str().unwrap_or("(Error fetching ISO path)")
        );
    } else {
        error_exit("Unable to get parent dir of iso temp folder.");
    }

    println!("Exiting now.");
}

/*
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
*/

#[must_use]
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

/*
let patches = vec![GamePatch {
    name: String::from("Force 1sec cutscenes"),
    offset: 0x4d5ac,
    offset_type: GamePatchOffsetType::Virtual,
    replacement_bytes: vec![0x90, 0x90, 0x90, 0x90, 0x90, 0x90],
    original_bytes: Some(vec![0xf3, 0x0f, 0x10, 0x3c, 0x24, 0x08]),
}];

let patches = vec![GamePatch {
    name: String::from("Remove celebration"),
    offset: 0x2bb36,
    offset_type: GamePatchOffsetType::Virtual,
    replacement_bytes: vec![0x90, 0x90, 0x90, 0x90, 0x90, 0x90],
    original_bytes: None,
}];
*/

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
