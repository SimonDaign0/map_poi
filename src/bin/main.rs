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
use esp_hal::{ Async, main };
use esp_hal::time::{ Duration, Instant, Rate };
use esp_println as _;
use esp_hal::i2c::master::{ I2c, Config as I2cConfig };
use ssd1306::{ I2CDisplayInterface, Ssd1306, prelude::* };

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
    }
}
esp_bootloader_esp_idf::esp_app_desc!();

use map_poi::utils::{
    fsm::{ StateMachine, Event }, //
    wrappers::{ Button, Coord },
};

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);
    //I2C
    let i2c_bus = I2c::new(
        _peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)) //100 to save power
    )
        .unwrap()
        .with_scl(_peripherals.GPIO1)
        .with_sda(_peripherals.GPIO0)
        .into_async();
    let interface = I2CDisplayInterface::new(i2c_bus);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0
    ).into_buffered_graphics_mode();
    display.init().expect("failed to init display");
    //
    let mut btns = [
        Button::new(_peripherals.GPIO3, 35),
        Button::new(_peripherals.GPIO2, 35),
        Button::new(_peripherals.GPIO5, 35),
        Button::new(_peripherals.GPIO6, 35),
        Button::new(_peripherals.GPIO7, 35),
    ]; //35ms debounce];

    let mut sm = StateMachine::new();
    sm.add_poi(Coord::new(12.0, 12.0));
    sm.add_poi(Coord::new(24.0, 24.0));
    loop {
        sm.render_map(&mut display);
        poll_btn(&mut sm, &mut btns);
        blocking_delay(10);
    }
}

fn blocking_delay(delay: u64) {
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(delay) {}
}

fn poll_btn(sm: &mut StateMachine, btns: &mut [Button<'_>; 5]) {
    for (index, btn) in btns.iter_mut().enumerate() {
        if btn.is_state_changed() {
            if btn.is_pressed() {
                sm.event_handler(Event::BtnPressed(index as u8));
            }
        }
        if btn.is_pressed() {
            sm.event_handler(Event::BtnContinuousPress(index as u8, btn.last_pressed));
        }
    }
}
