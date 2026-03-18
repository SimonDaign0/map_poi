#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::{ println };
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{ Duration, Instant };
use esp_println as _;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
    }
}
esp_bootloader_esp_idf::esp_app_desc!();

use map_poi::utils::wrappers::{ Button };

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);
    let mut btn = Button::new(_peripherals.GPIO2, 35); //35ms debounce

    loop {
        poll_btn(&mut btn);
        blocking_delay(10);
    }
}

fn blocking_delay(delay: u64) {
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(delay) {}
}

fn poll_btn(btn: &mut Button) {
    if btn.is_state_changed() {
        if btn.is_pressed() {
            println!("Button Pressed");
        } else {
            println!("Button Released");
        }
    }
}
