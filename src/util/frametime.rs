use std::collections::HashMap;
use std::time::{Instant, Duration};

pub struct Frametime {
    pub deltas: HashMap<String, Duration>,

    pub start_time: Instant,
    pub last_time: Instant,
}

impl Frametime {
    pub fn new() -> Frametime {
        Frametime {
            deltas: HashMap::new(),
        
            start_time: Instant::now(),
            last_time: Instant::now(),
        }
    }

    pub fn refresh(&mut self) {
        self.start_time = Instant::now();
        self.last_time = Instant::now();
        self.deltas.clear();
    }

    pub fn set(&mut self, s: &str) {
        let time_cur = Instant::now();
        let entry = self.deltas.entry(s.to_string());
        *entry.or_default() = time_cur - self.last_time;
        self.last_time = time_cur;
    }

    pub fn get_delta(&self) -> f32 {
        let time_cur = Instant::now();
        let delta = time_cur - self.start_time;

        delta.as_secs_f32()
    }
}

impl std::fmt::Display for Frametime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut sum_time: u128 = 0;
        self.deltas.iter().for_each(|entry| { sum_time += entry.1.as_millis(); });

        let mut display_text: String = String::from(format!("Frametime (total = {}): ", sum_time));

        for (segment, time) in &self.deltas {
            display_text.push_str(&format!("\n\t{}: {}", segment, time.as_millis()));
        }

        write!(f, "{}", display_text)
    }
}