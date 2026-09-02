# Oxide OS

Oxide OS is an experimental x86_64 operating system written primarily in Rust.

## Milestone 1: boot and print

The kernel uses the Rust `bootloader` crate to enter a freestanding `no_std`
x86_64 kernel. It writes a success message directly to the VGA text buffer and
then idles using the CPU `hlt` instruction. No Linux kernel or Linux runtime is
involved.

### Prerequisites

- Rust nightly with the `rust-src` component
- [`bootimage`](https://github.com/rust-osdev/bootimage): `cargo install bootimage`
- QEMU (`qemu-system-x86_64`)

### Run

```bash
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
cargo +nightly run
```

QEMU should display `Oxide OS` and `x86_64 kernel booted successfully.`

## Layout

- `src/main.rs`: early kernel entry point and CPU idle loop
- `src/vga.rs`: isolated early-console driver
- `x86_64-oxide.json`: freestanding target definition

Future hardware services will be added as separate modules so kernel entry
logic stays small.
