use super::wrappers::{ Map, Coord };
use defmt::println;
use ssd1306::{ mode::BufferedGraphicsMode, Ssd1306, prelude::* };
use esp_hal::i2c::master::I2c;
use esp_hal::Async;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Event {
    BtnPressed(u8),
    BtnContinuousPress(u8),
}

pub struct StateMachine {
    map: Map,
    zoom: i32,
}
impl StateMachine {
    pub fn new() -> Self {
        Self {
            map: Map::new(),
            zoom: 1,
        }
    }
    pub fn event_handler(&mut self, event: Event) {
        match event {
            Event::BtnPressed(4) => {
                self.zoom_out();
                println!("{}", self.zoom);
            }
            Event::BtnContinuousPress(index) => {
                let mut modx = 0.0;
                let mut mody = 0.0;
                match index {
                    0 => {
                        modx = 1.0 * (self.zoom as f64);
                    }
                    1 => {
                        modx = -1.0 * (self.zoom as f64);
                    }
                    2 => {
                        mody = 1.0 * (self.zoom as f64);
                    }
                    3 => {
                        mody = -1.0 * (self.zoom as f64);
                    }
                    _ => (),
                }

                let x = self.map.focus.x + modx;
                let y = self.map.focus.y + mody;
                self.map.focus = Coord::new(x, y);
            }
            _ => (),
        }
    }

    pub fn add_poi(&mut self, pos: Coord) {
        self.map.add_poi(pos);
    }

    pub fn zoom_out(&mut self) {
        self.zoom += 1;
    }

    pub fn render_map(
        &mut self,
        display: &mut Ssd1306<
            I2CInterface<I2c<'_, Async>>,
            DisplaySize128x64,
            BufferedGraphicsMode<DisplaySize128x64>
        >
    ) {
        self.map.render(display, self.zoom);
    }
}
