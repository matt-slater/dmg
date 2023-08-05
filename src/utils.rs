pub struct RingBuffer {
    cap: i16,
    len: i16,
    head: i16,
    tail: i16,
    data: [u8; 16],
}

impl RingBuffer {
    pub fn new(cap: i16) -> Self {
        RingBuffer {
            cap: cap,
            len: 0,
            head: 0,
            tail: -1,
            data: [0; 16],
        }
    }

    pub fn add(&mut self, e: u8) {
        if self.len == self.cap {
            println!("queue is full");
        } else {
            self.tail = (self.tail + 1) % self.cap;
            self.data[self.tail as usize] = e;
            self.len += 1;
        }
    }

    pub fn get(&mut self) -> u8 {
        if self.len == 0 {
            println!("empty");
            0
        } else {
            let tmp = self.data[self.head as usize];
            self.head = (self.head + 1) % self.cap;
            self.len -= 1;
            tmp
        }
    }

    pub fn clear(&mut self) {
        for e in self.data.iter_mut() {
            *e = 0;
        }
    }

    pub fn size(&self) -> i16 {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rb_add() {
        let mut rb = RingBuffer::new(2);

        rb.add(1);

        assert_eq!(1, rb.len);

        rb.add(2);

        assert_eq!(2, rb.len);

        assert_eq!(2, rb.cap);
    }

    #[test]
    fn test_rb_get() {
        let mut rb = RingBuffer::new(4);
        rb.add(1);
        rb.add(2);
        rb.add(3);
        rb.add(4);

        assert_eq!(1, rb.get());
        assert_eq!(2, rb.get());
        rb.add(5);
        assert_eq!(3, rb.get());
        assert_eq!(4, rb.get());
        rb.add(6);
        rb.add(7);
        assert_eq!(5, rb.get());
        assert_eq!(6, rb.get());
        assert_eq!(7, rb.get());
    }

    #[test]
    fn test_rb_clear() {
        let mut rb = RingBuffer::new(4);
        rb.add(1);
        rb.add(2);
        rb.add(3);
        rb.add(4);

        rb.clear();

        assert_eq!(rb.data[0], 0);
        assert_eq!(rb.data[1], 0);
        assert_eq!(rb.data[2], 0);
        assert_eq!(rb.data[3], 0);
    }
}
