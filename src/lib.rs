use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}, marker::PhantomData};

pub struct Blockfree<T> {
    version: Arc<AtomicUsize>,
    pointer: usize,
    phantom: PhantomData<T>
}

pub struct Replica<T> {
    version: Arc<AtomicUsize>,
    pointer: usize,
    phantom: PhantomData<T>
}

impl<T: Copy> Blockfree<T> {
    pub fn new(data: T) -> Self {
        let pointer = Box::into_raw(Box::new(data)) as usize;
        let version = Arc::new(AtomicUsize::new(0));
        version.store(0, Ordering::SeqCst);
        Self { version, pointer, phantom: PhantomData }
    }

    pub fn write(&mut self, data: T) {
        self.version.fetch_add(1, Ordering::SeqCst);
        let old_value = unsafe { *(self.pointer as *mut T) };
        unsafe { *(self.pointer as *mut T) = data };
        drop(old_value);
        self.version.fetch_add(1, Ordering::SeqCst);
    }

    pub fn replica(&self) -> Replica<T> {
        Replica {
            version: self.version.clone(),
            pointer: self.pointer.clone(),
            phantom: PhantomData
        }
    }
}


impl<T: Copy> Replica<T> {
    pub fn read(&self) -> Option<T> {
        let version = self.version.load(Ordering::SeqCst);
        let value = unsafe { *(self.pointer as *const T) };
        let version_after = self.version.load(Ordering::SeqCst);
        if version == version_after {
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
