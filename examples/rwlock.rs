#[cfg(not(any(target_arch = "aarch64",
              target_arch = "arm",
              target_arch = "powerpc",
              target_arch = "x86",
              target_arch = "x86_64")))]
fn main() {}

#[cfg(any(target_arch = "aarch64",
          target_arch = "arm",
          target_arch = "powerpc",
          target_arch = "x86",
          target_arch = "x86_64"))]
fn main() {
    use std::{time, thread};
    use std::sync::{Arc, RwLock};

    let mut threads = vec![];
    const NTHREADS: i32 = 100;
    const ITERATIONS: i32 = 100;

    let lock = Arc::new(RwLock::new(5));
    for _ in 0..NTHREADS {
      let lock = lock.clone();
      let t = thread::spawn(move ||
        for _ in 0..ITERATIONS {
          // sleep for 1 ms
          thread::sleep(time::Duration::from_millis(1));
          {
            let r1 = lock.read().unwrap();
            assert!(*r1 >= 5);

            match lock.try_read() {
              Ok(r2) => assert!(*r2 >= 5),
              Err(_) => {},
            }
          }

          {
            //let mut w = lock.try_write().unwrap();
            //*w += 1;
          }

          {
            let mut w = lock.write().unwrap();
            *w += 1;
          }
      });

      threads.push(t);
    }

    for t in threads {
        t.join().unwrap();
    }

    assert_eq!(*lock.read().unwrap(), 5 + NTHREADS * ITERATIONS);
}
