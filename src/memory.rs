//! Boot memory-map inspection and physical-frame allocation primitives.

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{FrameAllocator, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

const FRAME_SIZE: u64 = 4096;

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
///
/// Unlike the usual tutorial implementation, this allocator does not rebuild the entire
/// memory-map iterator and call `nth()` for every allocation. It walks each usable region
/// once, which makes early boot deterministic and avoids repeatedly scanning RAM metadata.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    region_index: usize,
    next_address: u64,
}

impl BootInfoFrameAllocator {
    /// Safety: the supplied map must be the bootloader's trustworthy memory map and each
    /// returned frame must be used at most once.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            region_index: 0,
            next_address: 0,
        }
    }

    fn next_usable_frame(&mut self) -> Option<PhysFrame> {
        while self.region_index < self.memory_map.len() {
            let region = &self.memory_map[self.region_index];

            if region.region_type != MemoryRegionType::Usable {
                self.region_index += 1;
                continue;
            }

            let start = region.range.start_addr();
            let end = region.range.end_addr();

            // Initialize the cursor at the first 4 KiB-aligned address in this region.
            if self.next_address < start {
                self.next_address = (start + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
            }

            if self.next_address >= end {
                self.region_index += 1;
                self.next_address = 0;
                continue;
            }

            let address = self.next_address;
            self.next_address = self.next_address.saturating_add(FRAME_SIZE);

            // Never hand physical frame zero to Rust code.
            if address == 0 {
                continue;
            }

            return Some(PhysFrame::containing_address(PhysAddr::new(address)));
        }

        None
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.next_usable_frame()
    }
}
