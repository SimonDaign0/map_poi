use super::wrappers::{ Map, Coord };
use defmt::println;
use ssd1306::{ mode::BufferedGraphicsMode, Ssd1306, prelude::* };
use esp_hal::i2c::master::I2c;
use esp_hal::Async;
use esp_hal::time::{ Instant, Duration };
use heapless::String;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub enum State {
    Navigating,
    PosCreation,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Event {
    BtnPressed(u8),
    BtnContinuousPress(u8, Instant),
    BtnReleased(u8),
}

pub struct StateMachine {
    map: Map,
    zoom: f32,
    state: State,
}
impl StateMachine {
    pub fn new() -> Self {
        Self {
            map: Map::new(),
            zoom: 1.0,
            state: State::Navigating,
        }
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn event_handler(&mut self, event: Event) {
        match self.state {
            State::Navigating => {
                match event {
                    Event::BtnPressed(_) => {
                        match event {
                            Event::BtnPressed(0) => {
                                self.shift_focus(Direction::Right);
                            }
                            Event::BtnPressed(1) => {
                                self.shift_focus(Direction::Left);
                            }
                            Event::BtnPressed(2) => {
                                self.shift_focus(Direction::Up);
                            }
                            Event::BtnPressed(3) => {
                                self.shift_focus(Direction::Down);
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
                                    self.shift_focus(Direction::Right);
                                }
                                Event::BtnContinuousPress(1, _) => {
                                    self.shift_focus(Direction::Left);
                                }
                                Event::BtnContinuousPress(2, _) => {
                                    self.shift_focus(Direction::Up);
                                }
                                Event::BtnContinuousPress(3, _) => {
                                    self.shift_focus(Direction::Down);
                                }
                                Event::BtnContinuousPress(4, _) => {
                                    self.set_state(State::PosCreation);
                                    self.add_poi(self.map.focus);
                                    println!("{}", self.map.pois);
                                }
                                _ => (),
                            }
                        }
                    }

                    _ => (),
                }
            }
            State::PosCreation => {
                match event {
                    Event::BtnReleased(4) => {
                        self.set_state(State::Navigating);
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn add_poi(&mut self, pos: Coord) {
        self.map.add_poi(pos);
    }

    pub fn add_crumb(&mut self, pos: Coord) {
        self.map.add_crumb(pos);
    }

    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 2.0).max(0.0625);
    }
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 2.0).min(128.0);
    }

    pub fn shift_focus(&mut self, direction: Direction) {
        match direction {
            Direction::Left => {
                self.map.focus.x -= 5.0 / (self.zoom as f64);
            }
            Direction::Right => {
                self.map.focus.x += 5.0 / (self.zoom as f64);
            }
            Direction::Up => {
                self.map.focus.y -= 5.0 / (self.zoom as f64);
            }
            Direction::Down => {
                self.map.focus.y += 5.0 / (self.zoom as f64);
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
