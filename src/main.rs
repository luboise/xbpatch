use std::{env, path::PathBuf};

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
                    if ret_args.iso_path.is_none() {
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
    let args = match parse_args(env::args()) {
        Ok(a) => a,
        Err(e) => match e {
            ArgError::InvalidArgState => {
                eprint!("Unable to proceed: Invalid arguments.");
                std::process::exit(1);
            }
            ArgError::IsoSpecifiedMultipleTimes => {
                eprint!("Unable to proceed: Iso has been specified multiple times.");
                std::process::exit(1);
            }
        },
    };

    dbg!(&args);
}

// Usage
// xbpatch gbtg.iso --config gbtg.xbconf
