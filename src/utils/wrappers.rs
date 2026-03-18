use esp_hal::gpio::{ Input, InputConfig, InputPin, Level, Output, OutputConfig, OutputPin, Pull };
use esp_hal::time::{ Duration, Instant };

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
