# blockfree

> Reading and writing without blocking

Can we read and write simultaneously without blocking?

_Warning: This is extremely sketchy and maybe incorrect!_

## Usage

```rust
use blockfree::Blockfree;

let mut blockfree = Blockfree::new(1);
let replica = blockfree.replica();

let handle = std::thread::spawn(move || {
    // Write from other threads without blocking reads
    blockfree.write(2);
});

handle.join().unwrap();
assert_eq!(replica.read(), Some(2));
```

## TODOs
- [ ] Handle `Drop` for `Blockfree`
- [ ] Create a version that caches the last successful read
- [ ] Switch SeqCst to least strict ordering
- [ ] Support Clone instead of just Copy inner types
- [ ] Add benchmarks
- [ ] Add tests for dropping