//! Boot memory-map inspection and physical-frame allocation primitives.

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{FrameAllocator, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

pub fn log_memory_map(memory_map: &MemoryMap) {
    let usable_bytes: u64 = memory_map
        .iter()
        .filter(|region| region.region_type == MemoryRegionType::Usable)
        .map(|region| region.range.end_addr() - region.range.start_addr())
        .sum();

    crate::kprintln!("Memory: {} usable KiB across the boot memory map.", usable_bytes / 1024);
}

/// Returns the active level-4 page table through the bootloader's physical-memory mapping.
///
/// Safety: `physical_memory_offset` must map all physical memory and the caller must not
/// create aliases that violate Rust's mutable-reference rules.
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_frame, _) = Cr3::read();
    let physical_address: PhysAddr = level_4_frame.start_address();
    let virtual_address = physical_memory_offset + physical_address.as_u64();
    let page_table_ptr: *mut PageTable = virtual_address.as_mut_ptr();
    unsafe { &mut *page_table_ptr }
}

/// A sequential allocator over the bootloader-marked usable physical frames.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Safety: the supplied map must be the bootloader's trustworthy memory map and each
    /// returned frame must be used at most once.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        Self { memory_map, next: 0 }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        self.memory_map
            .iter()
            .filter(|region| region.region_type == MemoryRegionType::Usable)
            .flat_map(|region| region.range.start_addr()..region.range.end_addr())
            .step_by(4096)
            .map(|address| PhysFrame::containing_address(PhysAddr::new(address)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
