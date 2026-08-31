#![no_std]
#![no_main]

pub mod afdt;
pub mod allocator;
pub mod bootargs;
pub mod macho;
pub mod uart;

pub mod main_defs {
    pub const RAM_BASE: u64 = 0x40000000;
    pub const L2_BLOCK: u64 = 0x200000;
    pub const XNU_LOAD_REGION: u64 = 0x42000000;
    pub const MACHO_SRC_ADDR: u64 = 0x48000000;
    pub const MEM_SIZE: u64 = 1024 * 1024 * 1024;
    pub const PANIC_LOG_SIZE: u32 = 0x80000;
    pub const PANIC_LOG_PHYS: u64 = RAM_BASE + MEM_SIZE - PANIC_LOG_SIZE as u64;

    pub const PL011_BASE: usize = 0x09000000;
    pub const GICD_BASE: u64 = 0x08000000;
    pub const GICC_BASE: u64 = 0x08010000;
    pub const PL031_BASE: u64 = 0x09010000;
    pub const RAMDISK_PHYS: u64 = 0x50000000;
    pub const RAMDISK_SIZE: u64 = 0x01000000; // 16 MB max window
}

use bootargs::XnuBootArguments;
use core::arch::global_asm;
use core::panic::PanicInfo;
use main_defs::*;

global_asm!(
    r#"
    .section .text._start, "ax", @progbits
    .global _start
_start:
    // Enable FP/SIMD (CPACR_EL1.FPEN = 0b11)
    mrs x0, cpacr_el1
    orr x0, x0, #(0x3 << 20)
    msr cpacr_el1, x0
    isb

    // Set up stack pointer
    ldr x0, =__stack_top
    mov sp, x0

    // Zero BSS
    ldr x0, =__bss_start
    ldr x1, =__bss_end
1:
    cmp x0, x1
    b.ge 2f
    str xzr, [x0], #8
    b 1b
2:
    // Jump to Rust main
    bl rust_main

    // If rust_main returns, halt
3:  wfe
    b 3b
"#
);

#[inline(always)]
fn round_up(v: u64, align: u64) -> u64 {
    (v + align - 1) & !(align - 1)
}

#[inline(never)]
unsafe fn jump_to_kernel(entry: u64, boot_args: u64) -> ! {
    core::arch::asm!(
        // Disable MMU & Caches before entering XNU
        "mrs x2, sctlr_el1",
        "bic x2, x2, #1",      // M: MMU disable (bit 0)
        "bic x2, x2, #4",      // C: Data cache disable (bit 2)
        "bic x2, x2, #4096",   // I: Instruction cache disable (bit 12)
        "msr sctlr_el1, x2",
        "isb",

        // Invalidate instruction cache
        "ic iallu",
        "isb",
        "dsb sy",

        // Branch to XNU kernel entry with x0 = boot_args
        "br x1",
        in("x1") entry,
        in("x0") boot_args,
        options(noreturn)
    );
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println!("xnu-qemu-boot (Rust): starting bootloader");

    let xnu_load_addr = XNU_LOAD_REGION + (0xfffffff007004000 % L2_BLOCK);

    let load_info = match unsafe { macho::load_macho_image(MACHO_SRC_ADDR, xnu_load_addr) } {
        Ok(info) => info,
        Err(e) => {
            println!("xnu-qemu-boot: fatal error loading Mach-O: {}", e);
            loop {
                core::hint::spin_loop();
            }
        }
    };

    let xnu_entry = load_info.entry - load_info.base + xnu_load_addr;
    let xnu_end = load_info.end - load_info.base + xnu_load_addr;

    println!(
        "  mach-o base=0x{:x} entry=0x{:x} end=0x{:x}",
        load_info.base, load_info.entry, load_info.end
    );
    println!("  xnu entry=0x{:x} xnu_end=0x{:x}", xnu_entry, xnu_end);

    let boot_args_addr = xnu_end;
    let boot_args_size = core::mem::size_of::<XnuBootArguments>() as u64;

    unsafe {
        let p = boot_args_addr as *mut u8;
        for i in 0..(boot_args_size as usize) {
            *p.add(i) = 0;
        }

        let boot_args = &mut *(boot_args_addr as *mut XnuBootArguments);

        boot_args.revision = 2;
        boot_args.version = 2;
        boot_args.virt_base = load_info.base;
        boot_args.phys_base = xnu_load_addr;
        boot_args.mem_size = MEM_SIZE - PANIC_LOG_SIZE as u64;
        boot_args.top_of_kernel_data = boot_args_addr + boot_args_size;

        // kdp_match_name=none: on arm64, kdp_init() configures *serial* KDP by
        // default, which shares the console UART. On panic the kernel then emits
        // KDP packet framing over that port (appearing as garbled text) and blocks
        // forever waiting for a remote debugger to attach -- which looks exactly
        // like a boot hang and destroys the panic message. Requesting a non-serial
        // debugger name makes kdp_init() bail out early, so panics print normally.
        let cmdline = "serial=3 serial-device-name=uart0 debug=0xa -v -noprogress fips_mode=8 rd=md0 cs_enforcement_disable=1 amfi_allow_any_signature=1 kdp_match_name=none";
        let cmd_bytes = cmdline.as_bytes();
        let copy_len = cmd_bytes.len().min(boot_args.command_line.len() - 1);
        for i in 0..copy_len {
            boot_args.command_line[i] = cmd_bytes[i];
        }

        let afdt_phys = boot_args.top_of_kernel_data;
        let afdt_len = afdt::build_afdt(
            afdt_phys,
            RAM_BASE,
            MEM_SIZE,
            PANIC_LOG_PHYS,
            PANIC_LOG_SIZE,
            RAMDISK_PHYS,
            RAMDISK_SIZE,
            cmdline,
        );

        boot_args.device_tree_p = afdt_phys - xnu_load_addr + load_info.base;
        boot_args.device_tree_length = afdt_len;
        boot_args.top_of_kernel_data = round_up(afdt_phys + afdt_len as u64, 0x10000);

        println!(
            "  boot_args=0x{:x} afdt=0x{:x} afdt_len={}",
            boot_args_addr, boot_args.device_tree_p, afdt_len
        );
        println!("  jumping into XNU kernel now...");

        jump_to_kernel(xnu_entry, boot_args_addr);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC: {:?}", info);
    loop {
        core::hint::spin_loop();
    }
}
