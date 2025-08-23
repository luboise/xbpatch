use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use crate::{
    HasPatches,
    memory::MemoryMap,
    patching::{Patch, PatchOffsetType},
};

#[derive(Debug)]
pub struct XBEWriter {
    xbe_file: File,
    xbe_header: XBEHeader,
    mem_map: MemoryMap,
}

#[derive(Debug, Default)]
pub struct PatchReport {
    successes: u32,
    failures: u32,
    total: u32,
}

impl PatchReport {
    #[inline]
    pub fn add_success(&mut self) {
        self.successes += 1;
    }
    #[inline]
    pub fn successes(&self) -> u32 {
        self.successes
    }

    #[inline]
    pub fn add_failure(&mut self) {
        self.failures += 1;
    }
    #[inline]
    pub fn failures(&self) -> u32 {
        self.failures
    }

    #[inline]
    pub fn total(&self) -> u32 {
        self.total
    }

    #[inline]
    pub fn patch_successful(&self) -> bool {
        self.failures == 0
    }

    pub fn increment_from_bool(&mut self, success: bool) {
        if success {
            self.successes += 1;
        } else {
            self.failures += 1;
        }
    }
}

impl XBEWriter {
    pub fn new(path: &PathBuf) -> Result<XBEWriter, std::io::Error> {
        let mut xbe_file = OpenOptions::new().read(true).write(true).open(&path)?;
        let xbe_header = XBEHeader::from_file(&mut xbe_file)?;
        let mem_map = MemoryMap::from_xbe_header(&xbe_header);

        dbg!(&mem_map);

        Ok(XBEWriter {
            xbe_file,
            mem_map,
            xbe_header,
        })
    }

    pub fn apply_patch(&mut self, patch: &Patch) -> Result<(), std::io::Error> {
        let offset: u64 = match patch.offset_type {
            PatchOffsetType::Raw => patch.offset.into(),
            PatchOffsetType::Virtual => self.mem_map.get_raw_offset(patch.offset)?.into(),
        };

        self.xbe_file.seek(SeekFrom::Start(offset))?;
        self.xbe_file.write_all(patch.replacement_bytes.as_ref())?;
        Ok(())
    }

    pub fn apply_patches<T: HasPatches>(
        &mut self,
        entry: &T,
    ) -> Result<PatchReport, std::io::Error> {
        let mut report = PatchReport::default();

        entry.get_patches().iter().for_each(|p| {
            // TODO: Remove these unwraps
            std::io::stdout().flush().expect("Unable to flush stdout");

            match self.apply_patch(p) {
                Ok(_) => {
                    report.successes += 1;
                }
                Err(_) => {
                    report.failures += 1;
                }
            };
        });

        Ok(report)
    }
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
