pub struct RingBuffer<T: Copy, const N: usize> {
    buffer: [Option<T>; N],
    head: usize,
    tail: usize,
    dropped: usize,
}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
    pub const fn new() -> Self {
        Self {
            buffer: [None; N],
            head: 0,
            tail: 0,
            dropped: 0,
        }
    }

    const fn is_full(&self) -> bool {
        (self.head + 1) % N == self.tail
    }

    /// Overwrites the oldest item
    pub const fn push_overwrite(&mut self, item: T) {
        if self.is_full() {
            self.tail = (self.tail + 1) % N; // Drop the oldest item
            self.dropped += 1;
        }
        // Can't fail, because we just made sure there is space
        let _ = self.try_push(item);
    }

    pub const fn try_push(&mut self, item: T) -> Result<(), T> {
        let next_head = (self.head + 1) % N;
        if next_head == self.tail {
            self.dropped += 1;
            return Err(item); // Buffer is full
        }
        self.buffer[self.head] = Some(item);
        self.head = next_head;
        Ok(())
    }

    pub const fn try_pop(&mut self) -> Option<T> {
        if self.head == self.tail {
            return None; // Buffer is empty
        }
        let item = self.buffer[self.tail];
        self.tail = (self.tail + 1) % N;
        item
    }

    pub const fn reset_drop_count(&mut self) -> usize {
        let d = self.dropped;
        self.dropped = 0;
        d
    }
}
