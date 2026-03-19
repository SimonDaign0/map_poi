#![no_std]
use core::fmt::{ self, write };

use defmt::{ Format, Formatter, println, write };
use embedded_graphics::image::ImageRaw;
use esp_hal::Async;
use esp_hal::gpio::{ Input, InputConfig, InputPin, Level, Output, OutputConfig, OutputPin, Pull };
use esp_hal::i2c::master::I2c;
use esp_hal::time::{ Duration, Instant };
use ssd1306::{ mode::BufferedGraphicsMode, Ssd1306, prelude::* };
use libm::{ round };
use heapless::String;

use embedded_graphics::{
    Drawable,
    Pixel,
    image::Image,
    mono_font::{ MonoTextStyleBuilder, ascii::FONT_6X10 },
    pixelcolor::BinaryColor,
    prelude::Point,
    text::{ Baseline, Text },
};

const RAW_POI: &[u8; 8] = &[0x08, 0x78, 0x78, 0x08, 0x08, 0x08, 0x3c, 0x7e];
pub const POI: ImageRaw<'_, BinaryColor> = ImageRaw::<BinaryColor>::new(RAW_POI, 8);
const DISPLAY_WIDTH: u8 = 128;
const DISPLAY_HEIGHT: u8 = 64;

pub struct Button<'d> {
    input: Input<'d>,
    last_state: bool, //true=>pressed, false=>released
    debounce: Duration,
    pub last_pressed: Instant,
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
}

impl Format for Coord {
    fn format(&self, fmt: Formatter) {
        write!(fmt, "({}, {})", self.x, self.y)
    }
}

pub fn is_inbound(local_x: f64, local_y: f64) -> bool {
    let is_inbound: bool =
        local_x >= 0.0 &&
        local_x < (DISPLAY_WIDTH as f64) &&
        local_y >= 0.0 &&
        local_y < (DISPLAY_HEIGHT as f64);
    is_inbound
}

pub struct Map {
    pub pois: [Option<Coord>; 10],
    pub crumbs: [Option<Coord>; 100],
    pub focus: Coord,
}
impl Map {
    pub fn new() -> Self {
        Self {
            pois: [None; 10],
            crumbs: [None; 100],
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

    pub fn add_crumb(&mut self, pos: Coord) {
        for slot in self.crumbs.iter_mut() {
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
        zoom: f32
    ) {
        display.clear_buffer();
        //POI
        for slot in self.pois {
            if let Some(poi) = slot {
                let local_x = (poi.x - self.focus.x) * (zoom as f64) + (DISPLAY_WIDTH as f64) / 2.0;
                let local_y =
                    (poi.y - self.focus.y) * (zoom as f64) + (DISPLAY_HEIGHT as f64) / 2.0;
                if is_inbound(local_x, local_y) {
                    let poi = Image::new(
                        &POI,
                        Point::new(round(local_x - 4.0) as i32, round(local_y - 4.0) as i32)
                    );
                    poi.draw(display).unwrap();
                }
            }
        }
        //CRUMB
        for slot in self.crumbs {
            if let Some(crumb) = slot {
                let local_x =
                    (crumb.x - self.focus.x) * (zoom as f64) + (DISPLAY_WIDTH as f64) / 2.0;
                let local_y =
                    (crumb.y - self.focus.y) * (zoom as f64) + (DISPLAY_HEIGHT as f64) / 2.0;
                if is_inbound(local_x, local_y) {
                    _ = Pixel(Point::new(local_x as i32, local_y as i32), BinaryColor::On).draw(
                        display
                    );
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
