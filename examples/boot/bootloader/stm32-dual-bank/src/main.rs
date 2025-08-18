#![no_std]
#![no_main]

mod fmt;

use core::{cell::RefCell, mem::MaybeUninit};

use cortex_m_rt::{entry, exception};

use embassy_boot_stm32::*;
use embassy_stm32::flash::{Flash, BANK1_REGION};
use embassy_stm32::{rcc::SupplyConfig, Config, SharedData};
use embassy_sync::blocking_mutex::Mutex;
use fmt::info;
// #[link_section = ".ram_d3"]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

// macro_rules! info {
//     ($s:literal $(, $x:expr)* $(,)?) => {
//         {
//             #[cfg(feature = "defmt")]
//             ::defmt::info!($s $(, $x)*);
//             #[cfg(not(feature="defmt"))]
//             let _ = ($( & $x ),*);
//         }
//     };
// }

#[entry]
fn main() -> ! {
    let mut config: Config = Default::default();
    config.rcc.supply_config = SupplyConfig::DirectSMPS;

    let p = embassy_stm32::init_primary(config, &SHARED_DATA);

    // Uncomment this if you are debugging the bootloader with debugger/RTT attached,
    // as it prevents a hard fault when accessing flash 'too early' after boot.

    for i in 0..10000000 {
        cortex_m::asm::nop();
    }
    info!("I'm here");
    let layout = Flash::new_blocking(p.FLASH).into_blocking_regions();
    let flash_bank1 = Mutex::new(RefCell::new(layout.bank1_region));
    let flash_bank2 = Mutex::new(RefCell::new(layout.bank2_region));

    let config = BootLoaderConfig::from_linkerfile_blocking(&flash_bank1, &flash_bank2, &flash_bank1);
    let active_offset = config.active.offset();
    info!("Active_offset: {}", active_offset);
    let bl = BootLoader::prepare::<_, _, _, 2048>(config);
    extern "C" {
        static __bootloader_active_start: u32;
        static __bootloader_active_end: u32;
        static __bootloader_state_start: u32;
        static __bootloader_dfu_start: u32;
    }

    let active_off = unsafe { core::ptr::addr_of!(__bootloader_active_start) as u32 };
    let state_off = unsafe { core::ptr::addr_of!(__bootloader_state_start) as u32 };
    let dfu_off = unsafe { core::ptr::addr_of!(__bootloader_dfu_start) as u32 };

    info!(
        "offs: active=0x{:x} state=0x{:x} dfu=0x{:x}",
        active_off, state_off, dfu_off
    );

    // Also print the *computed* absolute start the loader will jump to:
    let active_base = 0x0800_0000; // Bank1 base (for your own check)
    info!("active_abs ~= 0x{:08x}", active_base + active_off);

    unsafe { bl.load(BANK1_REGION.base + active_offset) }
}

#[no_mangle]
#[cfg_attr(target_os = "none", link_section = ".HardFault.user")]
unsafe extern "C" fn HardFault() {
    cortex_m::peripheral::SCB::sys_reset();
}

#[exception]
unsafe fn DefaultHandler(_: i16) -> ! {
    const SCB_ICSR: *const u32 = 0xE000_ED04 as *const u32;
    let irqn = core::ptr::read_volatile(SCB_ICSR) as u8 as i16 - 16;

    panic!("DefaultHandler #{:?}", irqn);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    cortex_m::asm::udf();
}
