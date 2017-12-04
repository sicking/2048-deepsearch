extern crate std;

use std::collections::VecDeque;

#[derive(Debug)]
pub struct MovAvg<T> {
  total: T,
  vals: VecDeque<T>,
}

impl<T: std::ops::Add<Output=T> +
        std::ops::Sub<Output=T> +
        std::ops::Div<Output=T> +
        std::convert::From<i32> +
        std::marker::Copy> MovAvg<T> {
  pub fn new() -> MovAvg<T> {
    MovAvg{ total: T::from(0i32), vals: VecDeque::new() }
  }

  pub fn init(&mut self, n: usize) {
    for _ in 0..n {
      self.vals.push_back(T::from(0i32));
    }
  }

  pub fn add(&mut self, val: T) {
    self.vals.push_back(val);
    self.total = self.total + val;
  }

  pub fn drop(&mut self) {
    if let Some(val) = self.vals.pop_front() {
      self.total = self.total - val;
    }
  }

  pub fn avg(&self) -> T {
    self.total / T::from(self.vals.len() as i32)
  }
}

