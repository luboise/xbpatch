use std::{env, path::PathBuf, process::Command};

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

    let iso: PathBuf = args.iso_path.take().expect("Expected an iso path.");

    let output = if cfg!(target_os = "windows") {
        todo!();
        // Command::new("cmd")
        //     .args(["/C", "echo hello"])
        //     .output()
        //     .expect("failed to execute process")
    } else {
        Command::new("extract-xiso")
            .arg("-x")
            .arg(&iso)
            .arg("-d")
            .arg("./xbpatch_temp")
            .output()
            .expect("failed to extract iso")
    };

    if !output.stderr.is_empty() {
        let iso_string = iso
            .into_os_string()
            .into_string()
            .expect("Unable to get OsString from iso path.");
        let msg: String = format!("Unable to extract {}", iso_string);
        let desc = str::from_utf8(&output.stderr).expect("ISO path is not UTF-8 path.");

        error_exit_with_details(msg, desc);
    }

    dbg!(&str::from_utf8(&output.stdout));
    dbg!(&str::from_utf8(&output.stderr));
}

// Usage
// xbpatch gbtg.iso --config gbtg.xbconf
