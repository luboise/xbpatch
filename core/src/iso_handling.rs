use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

pub fn extract_iso(
    extract_xiso_path: &Path,
    iso_path: &PathBuf,
    extraction_path: &PathBuf,
) -> Result<(), std::io::Error> {
    let cwd = extraction_path.parent().unwrap();

    if !extraction_path.exists() {
        std::fs::create_dir_all(&cwd)?;
    }

    // If the folder doesn't exist (or the user just deleted it), then extract the game
    let mut extractor = Command::new(extract_xiso_path)
        .current_dir(cwd)
        .arg("-x")
        .arg(iso_path)
        // .arg("-d")
        // .arg(extraction_path)
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

    // #[cfg(target_family = "windows")]
    // {
    //     let iso_basename = iso_path
    //         .file_stem()
    //         .expect("Unable to get basename from iso.");

    //     let from_dir = cwd.join(iso_basename);
    //     let to_dir = extraction_path;

    //     println!(
    //         "\n\nCopying from {} to {}",
    //         from_dir.display(),
    //         to_dir.display()
    //     );

    //     fs::copy(from_dir, to_dir).map_err(|e| {
    //         std::io::Error::other(format!("Failed to extract folder. Error: {}", e))
    //     })?;
    // }

    Ok(())
}

pub fn create_iso(
    extract_xiso_path: &Path,
    iso_path: &PathBuf,
    iso_files_path: &PathBuf,
) -> Result<(), std::io::Error> {
    // If the folder doesn't exist (or the user just deleted it), then extract the game

    // if cfg!(target_os = "windows") {
    let mut extractor = Command::new(&extract_xiso_path)
        .arg("-c")
        .arg(&iso_files_path)
        .arg(&iso_path)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start extract-xiso");

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

    Ok(())
}

