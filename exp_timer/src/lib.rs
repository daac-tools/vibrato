use std::time::Instant;

// From https://github.com/jermp/essentials/blob/master/include/essentials.hpp
pub struct Timer {
    times: Vec<f64>,
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            times: vec![],
            start: Instant::now(),
        }
    }

    pub fn start(&mut self) {
        self.start = Instant::now();
    }

    pub fn stop(&mut self) {
        self.times.push(self.start.elapsed().as_secs_f64());
    }

    pub fn runs(&self) -> usize {
        self.times.len()
    }

    pub fn reset(&mut self) {
        self.times.clear();
    }

    pub fn min(&self) -> f64 {
        self.times.iter().cloned().reduce(f64::min).unwrap()
    }

    pub fn max(&self) -> f64 {
        self.times.iter().cloned().reduce(f64::max).unwrap()
    }

    pub fn discard_min(&mut self) {
        assert!(!self.times.is_empty());
        let (idx, _) = self
            .times
            .iter()
            .cloned()
            .enumerate()
            .min_by(|(_, x), (_, y)| x.partial_cmp(y).unwrap())
            .unwrap();
        self.times.remove(idx);
    }

    pub fn discard_max(&mut self) {
        assert!(!self.times.is_empty());
        let (idx, _) = self
            .times
            .iter()
            .cloned()
            .enumerate()
            .min_by(|(_, x), (_, y)| y.partial_cmp(x).unwrap())
            .unwrap();
        self.times.remove(idx);
    }

    pub fn total(&self) -> f64 {
        self.times.iter().fold(0.0, |acc, &x| acc + x)
    }

    pub fn average(&self) -> f64 {
        self.total() / self.runs() as f64
    }
}
