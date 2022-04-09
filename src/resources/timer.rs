#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Timer(f32);

impl Timer {
    pub fn new() -> Self {
        Self(0.0)
    }

    pub fn add(&mut self, time_to_add: f32) {
        self.0 += time_to_add;
    }

    pub fn get_time_string(&self) -> String {
        let seconds = (self.0 / 1000.0) as u64 % 60;
        let minutes = ((self.0 / 1000.0) as u64 / 60) % 60;
        let seconds_string = if seconds < 10 {
            String::from("0") + &seconds.to_string()
        } else {
            seconds.to_string()
        };
        let minutes_string = if minutes < 10 {
            String::from("0") + &minutes.to_string()
        } else {
            minutes.to_string()
        };
        format!("{}:{}", minutes_string, seconds_string)
    }
}
