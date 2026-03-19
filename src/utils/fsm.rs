use super::wrappers::{ Map, Coord };
use defmt::println;
use ssd1306::{ mode::BufferedGraphicsMode, Ssd1306, prelude::* };
use esp_hal::i2c::master::I2c;
use esp_hal::Async;
use esp_hal::time::{ Instant, Duration };

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Event {
    BtnPressed(u8),
    BtnContinuousPress(u8, Instant),
}

pub struct StateMachine {
    map: Map,
    zoom: f32,
}
impl StateMachine {
    pub fn new() -> Self {
        Self {
            map: Map::new(),
            zoom: 1.0,
        }
    }
    pub fn event_handler(&mut self, event: Event) {
        match event {
            Event::BtnPressed(_) => {
                match event {
                    Event::BtnPressed(0) => {
                        self.shift_focus(Direction::Right, 5.0);
                    }
                    Event::BtnPressed(1) => {
                        self.shift_focus(Direction::Left, 5.0);
                    }
                    Event::BtnPressed(2) => {
                        self.shift_focus(Direction::Up, 5.0);
                    }
                    Event::BtnPressed(3) => {
                        self.shift_focus(Direction::Down, 5.0);
                    }
                    Event::BtnPressed(4) => {
                        self.zoom_out();
                    }
                    _ => (),
                }
                println!("zoom {}", self.zoom);
            }
            Event::BtnContinuousPress(_, first_press) => {
                if first_press.elapsed() > Duration::from_millis(500) {
                    match event {
                        Event::BtnContinuousPress(0, _) => {
                            self.shift_focus(Direction::Right, 3.0);
                        }
                        Event::BtnContinuousPress(1, _) => {
                            self.shift_focus(Direction::Left, 3.0);
                        }
                        Event::BtnContinuousPress(2, _) => {
                            self.shift_focus(Direction::Up, 3.0);
                        }
                        Event::BtnContinuousPress(3, _) => {
                            self.shift_focus(Direction::Down, 3.0);
                        }
                        _ => (),
                    }
                }
            }

            _ => (),
        }
    }

    pub fn add_poi(&mut self, pos: Coord) {
        self.map.add_poi(pos);
    }

    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 2.0).max(0.0625);
    }
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 2.0).min(128.0);
    }

    pub fn shift_focus(&mut self, direction: Direction, amt: f64) {
        match direction {
            Direction::Left => {
                self.map.focus.x -= amt;
            }
            Direction::Right => {
                self.map.focus.x += amt;
            }
            Direction::Up => {
                self.map.focus.y -= amt;
            }
            Direction::Down => {
                self.map.focus.y += amt;
            }
        }
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
