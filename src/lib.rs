use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}, marker::PhantomData};

pub struct Blockfree<T> {
    pub pointer: Arc<AtomicUsize>,
    phantom: PhantomData<T>,
}

pub struct Replica<T> {
    pub pointer: Arc<AtomicUsize>,
    phantom: PhantomData<T>,
}

impl<T> Blockfree<T> {
    pub fn new(data: T) -> Self {
        let pointer = Box::into_raw(Box::new(data));
        let atomic_pointer = Arc::new(AtomicUsize::new(0));
        atomic_pointer.store(pointer as usize, Ordering::SeqCst);
        Self { pointer: atomic_pointer, phantom: PhantomData }
    }

    pub fn write(&mut self, data: T) {
        let pointer = Box::into_raw(Box::new(data));
        let old_pointer = self.pointer.load(Ordering::SeqCst);
        self.pointer.store(pointer as usize, Ordering::SeqCst);
        drop(unsafe { Box::from_raw(old_pointer as *mut T) });
    }

    pub fn replica(&self) -> Replica<T> {
        Replica {
            pointer: self.pointer.clone(),
            phantom: PhantomData,
        }
    }
}


impl<T: Copy> Replica<T> {
    pub fn read(&self) -> Option<T> {
        let pointer = self.pointer.load(Ordering::SeqCst);
        let value = unsafe { *(pointer as *const T) };
        let pointer_after = self.pointer.load(Ordering::SeqCst);
        if pointer == pointer_after {
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_immediately() {
        let blockfree = Blockfree::new(1);
        let replica = blockfree.replica();
        assert_eq!(replica.read(), Some(1));
    }

    #[test]
    fn read_after_set() {
        let mut blockfree = Blockfree::new(1);
        let replica = blockfree.replica();
        blockfree.write(2);
        assert_eq!(replica.read(), Some(2));
    }

    #[test]
    fn test_multithreaded() {
        let mut blockfree = Blockfree::new(1);
        let replica = blockfree.replica();
        let handle = std::thread::spawn(move || {
            blockfree.write(2);
        });
        handle.join().unwrap();
        assert_eq!(replica.read(), Some(2));
    }
    
    #[test]
    fn test_string_slices() {
        let mut blockfree = Blockfree::new("hello");
        let replica = blockfree.replica();
        let handle = std::thread::spawn(move || {
            blockfree.write("world");
        });
        handle.join().unwrap();
        assert_eq!(replica.read(), Some("world"));
    }
}
