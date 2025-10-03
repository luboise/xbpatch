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

// #[must_use]
pub fn backup_file(filepath: &PathBuf) -> Result<PathBuf, std::io::Error> {
    let mut new_filepath: PathBuf = filepath.clone();

    if let Some(filename) = filepath.file_name() {
        let mut backup_name = filename.to_os_string();
        backup_name.push(".bak");
        new_filepath.set_file_name(backup_name);

        // Do not overwrite if it exists
        // TODO: Make this an option or prompt to the user
        if new_filepath.exists() {
            println!("Restoring backup default.xbe...");
            std::fs::copy(&new_filepath, filepath)?;
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

// #[must_use]
pub fn restore_backup(original_path: &PathBuf) -> Result<Option<PathBuf>, std::io::Error> {
    let mut backup_path: PathBuf = original_path.clone();

    if let Some(filename) = original_path.file_name() {
        let mut backup_name = filename.to_os_string();
        backup_name.push(".bak");
        backup_path.set_file_name(backup_name);

        // Do not overwrite if it exists
        // TODO: Make this an option or prompt to the user
        if !backup_path.exists() {
            return Ok(None);
        }

        match std::fs::copy(&backup_path, original_path) {
            Ok(_val) => Ok(Some(original_path.clone())),
            Err(e) => Err(e),
        }
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "File has no filename",
        ));
    }
}
