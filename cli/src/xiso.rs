use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom},
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

pub fn extract_iso(iso_path: &PathBuf, extraction_path: &PathBuf) -> Result<(), std::io::Error> {
    // If the folder doesn't exist (or the user just deleted it), then extract the game
    if cfg!(target_os = "windows") {
        todo!();
    } else {
        let mut extractor = Command::new("extract-xiso")
            .arg("-x")
            .arg(&iso_path)
            .arg("-d")
            .arg(&extraction_path)
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
    };

    Ok(())
}

pub fn create_iso(iso_path: &PathBuf, iso_files_path: &PathBuf) -> Result<(), std::io::Error> {
    // If the folder doesn't exist (or the user just deleted it), then extract the game
    if cfg!(target_os = "windows") {
        todo!();
    } else {
        let mut extractor = Command::new("extract-xiso")
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
    };

    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct XBESectionHeader {
    pub flags: u32,
    pub virtual_offset: u32,
    pub virtual_size: u32,
    pub file_offset: u32,
    pub file_size: u32,
    pub name_ptr: u32,
    pub reference: u32,
    pub head_ref_ptr: u32,
    pub tail_ref_ptr: u32,
}

#[derive(Debug)]
pub struct XBEHeader {
    image_base: u32,

    // Number of memory sections
    section_count: u32,
    // Address to the first section
    section_header_ptr: u32,

    sections: Vec<XBESectionHeader>,
}

impl XBEHeader {
    pub fn sections(&self) -> &Vec<XBESectionHeader> {
        &self.sections
    }

    pub fn section_count(&self) -> u32 {
        self.section_count
    }

    pub fn from_file(file: &mut File) -> Result<XBEHeader, std::io::Error> {
        file.seek(SeekFrom::Start(0x104))?;

        let mut buf_u32 = [0u8; 4];

        file.read_exact(&mut buf_u32)?;
        let image_base = u32::from_le_bytes(buf_u32);

        file.seek(SeekFrom::Start(0x11c))?;
        file.read_exact(&mut buf_u32)?;
        let section_count = u32::from_le_bytes(buf_u32);

        file.read_exact(&mut buf_u32)?;
        let section_header_ptr = u32::from_le_bytes(buf_u32);

        let mut sections = Vec::new();
        sections.resize(section_count as usize, XBESectionHeader::default());

        file.seek(SeekFrom::Start((section_header_ptr - image_base).into()))?;
        for i in 0..section_count as usize {
            file.read_exact(&mut buf_u32)?;
            sections[i].flags = u32::from_le_bytes(buf_u32);

            file.read_exact(&mut buf_u32)?;
            sections[i].virtual_offset = u32::from_le_bytes(buf_u32);

            file.read_exact(&mut buf_u32)?;
            sections[i].virtual_size = u32::from_le_bytes(buf_u32);

            file.read_exact(&mut buf_u32)?;
            sections[i].file_offset = u32::from_le_bytes(buf_u32);

            file.read_exact(&mut buf_u32)?;
            sections[i].file_size = u32::from_le_bytes(buf_u32);

            file.read_exact(&mut buf_u32)?;
            sections[i].name_ptr = u32::from_le_bytes(buf_u32);

            file.read_exact(&mut buf_u32)?;
            sections[i].reference = u32::from_le_bytes(buf_u32);

            file.read_exact(&mut buf_u32)?;
            sections[i].head_ref_ptr = u32::from_le_bytes(buf_u32);

            file.read_exact(&mut buf_u32)?;
            sections[i].tail_ref_ptr = u32::from_le_bytes(buf_u32);
        }

        Ok(XBEHeader {
            image_base,
            section_count,
            section_header_ptr,
            sections,
        })
    }
}
