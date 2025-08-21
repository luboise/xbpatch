pub struct Patch {
    name: std::String,
    offset: u32,
    new_value: Vec<u32>,
}

pub struct MemoryMapping {
    mem_offset: usize,
    size: usize,
    file_offset: usize,
}
