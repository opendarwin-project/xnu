#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct VideoInformation {
    pub base_addr: u64,
    pub display: u64,
    pub bytes_per_row: u64,
    pub width: u64,
    pub height: u64,
    pub depth: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct XnuBootArguments {
    pub revision: u16,
    pub version: u16,
    pub _pad0: u32,
    pub virt_base: u64,
    pub phys_base: u64,
    pub mem_size: u64,
    pub top_of_kernel_data: u64,
    pub video: VideoInformation,
    pub machine_type: u32,
    pub _pad1: u32,
    pub device_tree_p: u64,
    pub device_tree_length: u32,
    pub command_line: [u8; 1024],
    pub _pad2: u32,
    pub boot_flags: u64,
    pub mem_size_actual: u64,
}

impl Default for XnuBootArguments {
    fn default() -> Self {
        Self {
            revision: 0,
            version: 0,
            _pad0: 0,
            virt_base: 0,
            phys_base: 0,
            mem_size: 0,
            top_of_kernel_data: 0,
            video: VideoInformation::default(),
            machine_type: 0,
            _pad1: 0,
            device_tree_p: 0,
            device_tree_length: 0,
            command_line: [0u8; 1024],
            _pad2: 0,
            boot_flags: 0,
            mem_size_actual: 0,
        }
    }
}
