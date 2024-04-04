use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Value {
    total: i32,
    current: i32,
}

impl Value {
    pub fn new(total: u32) -> Self {
        Value {
            total: total as i32,
            current: total as i32,
        }
    }

    pub fn total(&self) -> i32 {
        self.total
    }

    pub fn reduce(&mut self, n: u32) -> i32 {
        let n = n as i32;
        self.current -= n;
        if self.current < 0 {
            let rest = -self.current;
            self.current = 0;
            rest
        } else {
            0
        }
    }

    pub fn add(&mut self, n: u32) -> i32 {
        let n = n as i32;
        self.current += n;
        if self.current > self.total {
            let rest = self.current - self.total;
            self.current = self.total;
            rest
        } else {
            0
        }
    }

    pub fn empty(&mut self) {
        self.current = 0;
    }

    pub fn reset(&mut self) {
        self.current = self.total;
    }
}

impl Deref for Value {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.current
    }
}

impl DerefMut for Value {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.current
    }
}

#[cfg(test)]
mod testing_values {
    use super::Value;

    #[test]
    fn value_reduce_tests() {
        let mut v = Value::new(10);
        assert_eq!(v.total(), 10);
        assert_eq!(*v, 10);
        let o = v.reduce(5);
        assert_eq!(*v, 5);
        assert_eq!(o, 0);
        let o = v.reduce(6);
        assert_eq!(*v, 0);
        assert_eq!(o, 1);
    }

    #[test]
    fn value_add_tests() {
        let mut v = Value::new(10); // 10/10
        assert_eq!(v.total(), 10);
        assert_eq!(*v, 10);
        let o = v.add(5);                   // (10 + 5)/10 = 10 carry 5
        assert_eq!(*v, v.total());
        assert_eq!(o, 5);
        let _ = v.reduce(5);                     // 10 - 5 = 5/10
        let o = v.add(6);                   // (5 + 6)/10 = 10 carry 1
        assert_eq!(*v, v.total());
        assert_eq!(o, 1);
    }
}