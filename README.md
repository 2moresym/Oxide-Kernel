# Oxide OS

Oxide OS is an experimental x86_64 operating system written primarily in Rust.

## Current milestone: diagnostics and memory foundations

The freestanding `no_std` kernel boots through the Rust `bootloader` crate and:

- mirrors kernel logs to VGA text mode and QEMU's COM1 serial port;
- loads a GDT plus an exception IDT, including a dedicated double-fault stack;
- reports breakpoint and page-fault diagnostics;
- reads the bootloader memory map, exposes the active level-4 page table, and
  creates a safe-to-use-next `BootInfoFrameAllocator` for usable 4 KiB frames.

Hardware interrupts deliberately remain disabled until the PIC/APIC driver is
introduced. No Linux kernel or Linux runtime is involved.

## Build and run

```bash
rustup toolchain install nightly --profile minimal
rustup component add rust-src llvm-tools-preview --toolchain nightly
cargo install bootimage
cargo +nightly run
```

The QEMU window shows VGA output. To also show serial logs in the launching
terminal, use:

```bash
cargo +nightly run -- -serial stdio
```

## Layout

- `src/console.rs`: unified kernel logging API
- `src/serial.rs`: COM1 serial driver for QEMU diagnostics
- `src/cpu/`: GDT and exception-IDT initialization
- `src/memory.rs`: memory-map inspection, page-table access, frame allocation
- `src/vga.rs`: VGA text-mode driver

The next milestone is physical-frame allocation ownership and safe page mapping.
