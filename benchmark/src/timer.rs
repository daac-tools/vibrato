// Cloned from https://github.com/jermp/essentials/blob/master/include/essentials.hpp
//
// Copyright 2019 - 2021 Giulio Ermanno Pibiri
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR
// OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
// ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.

use std::time::Instant;

pub struct Timer {
    times: Vec<f64>,
    start: Instant,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
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
