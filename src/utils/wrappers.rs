#![no_std]
use defmt::println;
use esp_hal::Async;
use esp_hal::gpio::{ Input, InputConfig, InputPin, Level, Output, OutputConfig, OutputPin, Pull };
use esp_hal::i2c::master::I2c;
use esp_hal::time::{ Duration, Instant };
use ssd1306::{ mode::BufferedGraphicsMode, Ssd1306, prelude::* };
use libm::{ floor };

use embedded_graphics::{
    Drawable,
    Pixel,
    image::Image,
    mono_font::{ MonoTextStyleBuilder, ascii::FONT_6X10 },
    pixelcolor::BinaryColor,
    prelude::Point,
    text::{ Baseline, Text },
};

pub struct Button<'d> {
    input: Input<'d>,
    last_state: bool, //true=>pressed, false=>released
    debounce: Duration,
    last_pressed: Instant,
}
impl<'d> Button<'d> {
    pub fn new(pin: impl InputPin + 'd, debounce: u64) -> Self {
        let input = Input::new(pin, InputConfig::default().with_pull(Pull::Up));
        Self {
            input,
            last_state: false,
            debounce: Duration::from_millis(debounce), //debounce set to 35ms to detect press state changes
            last_pressed: Instant::now(),
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.input.is_low()
    }

    pub fn is_state_changed(&mut self) -> bool {
        let current = self.is_pressed();
        //Debounce handler
        if current != self.last_state {
            if self.last_pressed.elapsed() < self.debounce {
                return false;
            }
            self.last_pressed = Instant::now();
            self.last_state = current;
            return true;
        }
        false
    }
}

pub struct Led<'d> {
    output: Output<'d>,
}
impl<'d> Led<'d> {
    pub fn new(pin: impl OutputPin + 'd) -> Self {
        Self {
            output: Output::new(pin, Level::Low, OutputConfig::default()),
        }
    }
    pub fn on(&mut self) {
        self.output.set_high();
    }
    pub fn off(&mut self) {
        self.output.set_low();
    }
}

#[derive(Clone, Copy)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
}
impl Coord {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
        }
    }
    pub fn is_inbound(&self, focus: Coord, width: i32, height: i32) -> bool {
        let (min_x, min_y) = (floor(focus.x) as i32, floor(focus.y) as i32);
        let (x, y) = (floor(self.x) as i32, floor(self.y) as i32);
        let is_inbound: bool = x >= min_x && x < min_x + width && y >= min_y && y < min_y + height;
        is_inbound
    }
}

pub struct Map {
    pois: [Option<Coord>; 10],
    pub focus: Coord,
}
impl Map {
    pub fn new() -> Self {
        Self {
            pois: [None; 10],
            focus: Coord::new(0.0, 0.0),
        }
    }

    pub fn add_poi(&mut self, pos: Coord) {
        for slot in self.pois.iter_mut() {
            if slot.is_none() {
                *slot = Some(Coord::new(pos.x, pos.y));
                break;
            }
        }
    }

    pub fn render(
        &self,
        display: &mut Ssd1306<
            I2CInterface<I2c<'_, Async>>,
            DisplaySize128x64,
            BufferedGraphicsMode<DisplaySize128x64>
        >,
        zoom: i32
    ) {
        let focus = Coord::new(
            self.focus.x - (128 as f64) / (zoom as f64),
            self.focus.y - (64 as f64) / (zoom as f64)
        );
        display.clear_buffer();
        for slot in self.pois {
            if let Some(poi) = slot {
                let local_coord = Coord::new(poi.x / (zoom as f64), poi.y / (zoom as f64));
                if poi.is_inbound(self.focus, 128 * zoom, 64 * zoom) {
                    let local_x = floor(local_coord.x - focus.x) as i32;
                    let local_y = floor(local_coord.y - focus.y) as i32;
                    _ = Pixel(Point::new(local_x, local_y), BinaryColor::On).draw(display);
                }
            }
        }
        display.flush().expect("failed to flush poi");
    }
    /*pub fn render(
        &self,
        display: &mut Ssd1306<
            I2CInterface<I2c<'_, Async>>,
            DisplaySize128x64,
            BufferedGraphicsMode<DisplaySize128x64>
        >
    ) {
        display.clear_buffer();
        for slot in self.pois {
            if let Some(poi) = slot {
                if poi.is_inbound(self.focus, 128, 64) {
                    let local_x = floor(poi.x - self.focus.x) as i32;
                    let local_y = floor(poi.y - self.focus.y) as i32;
                    _ = Pixel(Point::new(local_x, local_y), BinaryColor::On).draw(display);
                }
            }
        }
        display.flush().expect("failed to flush poi");
    } */
}
