use std::collections::HashMap;
use std::time::{Instant, Duration};

pub struct Frametime {
    pub deltas: HashMap<String, Duration>,

    pub last_time: Instant,
}

impl Frametime {
    pub fn new() -> Frametime {
        Frametime {
            deltas: HashMap::new(),
        
            last_time: Instant::now(),
        }
    }

    pub fn refresh(&mut self) {
        self.last_time = Instant::now();
        self.deltas.clear();
    }

    pub fn set(&mut self, s: &str) {
        let time_cur = Instant::now();
        let entry = self.deltas.entry(s.to_string());
        *entry.or_default() = time_cur - self.last_time;
        self.last_time = time_cur;
    }
}

impl std::fmt::Display for Frametime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut display_text: String = String::from("Frametime: ");

        for (segment, time) in &self.deltas {
            display_text.push_str(&format!("\n\t{}: {}", segment, time.as_millis()));
        }

        write!(f, "{}", display_text)
    }
}