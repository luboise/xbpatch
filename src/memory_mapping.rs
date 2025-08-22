use crate::xiso::XBEHeader;

#[derive(Debug)]
pub struct MemoryMap {
    mappings: Vec<MemoryMapping>,
}

#[derive(Debug)]
pub struct MemoryMapping {
    pub file_start: u32,
    pub virtual_start: u32,
    pub size: u32,
}

impl MemoryMap {
    pub fn from_xbe_header(header: &XBEHeader) -> MemoryMap {
        let mut mappings = Vec::new();

        for section in header.sections() {
            mappings.push(MemoryMapping {
                file_start: section.file_offset,
                virtual_start: section.virtual_offset,
                size: section.virtual_size,
            });
        }

        MemoryMap { mappings }
    }

    pub fn get_raw_offset(&self, address: u32) -> Result<u32, std::io::Error> {
        for mapping in &self.mappings {
            if mapping.virtual_start <= address && address <= mapping.virtual_start + mapping.size {
                return Ok(mapping.file_start + address - mapping.virtual_start);
            }
        }

        Err(std::io::Error::other(
            "The memory map does not contain the virtual address specified.",
        ))
    }

    pub fn new(mappings: Vec<MemoryMapping>) -> MemoryMap {
        MemoryMap { mappings }
    }
}

impl MemoryMapping {
    fn contains_block(&self, block_start: u32, block_size: u32) -> bool {
        block_start < self.size && (self.size - block_start - block_size) < self.size
    }
}

#[cfg(test)]
mod tests {
    use crate::memory_mapping::{MemoryMap, MemoryMapping};

    #[test]
    fn get_raw_address_tests() -> Result<(), String> {
        let mem = get_mem_map();

        assert_eq!(mem.get_raw_offset(0x4d5ac).unwrap(), 0x3d5ac);
        Ok(())
    }

    fn get_mem_map() -> MemoryMap {
        MemoryMap {
            mappings: vec![
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
            ],
        }
    }
}
