#[cfg(not(any(target_arch = "aarch64",
              target_arch = "arm",
              target_arch = "powerpc",
              target_arch = "x86",
              target_arch = "x86_64")))]
fn main() {}

fn test_mutex() {
  use std::{time, thread};
  use std::sync::{Arc, Mutex};

  let mut threads = vec![];
  const NTHREADS: i32 = 100;
  const ITERATIONS: i32 = 100;

  let data = Arc::new(Mutex::new(5));
  for _ in 0..NTHREADS {
    let data = data.clone();
    let t = thread::spawn(move ||
      for _ in 0..ITERATIONS {
        // sleep for 1 ms
        thread::sleep(time::Duration::from_millis(1));

        let mut data = data.lock().unwrap();
        *data += 1;

        match data.try_lock() {
          Ok(_) => panic!("cannot re-lock non-reentrant mutex"),
          Err(_) => {},
        }
    });

    threads.push(t);
  }

  for t in threads {
      t.join().unwrap();
  }

  assert_eq!(*data.lock().unwrap(), 5 + NTHREADS * ITERATIONS);
}

fn test_reentrant_mutex() {
  use std::{time, thread};
  use std::sync::{Arc, Mutex};

  let mut threads = vec![];
  const NTHREADS: i32 = 100;
  const ITERATIONS: i32 = 100;

  let data = Arc::new(Mutex::new(5));
  for _ in 0..NTHREADS {
    let data = data.clone();
    let t = thread::spawn(move ||
      for _ in 0..ITERATIONS {
        // sleep for 1 ms
        thread::sleep(time::Duration::from_millis(1));

        let mut data = data.lock().unwrap();
        *data += 1;

        let mut data2 = data.lock().unwrap();
        *data2 += 1;

        match data.try_lock() {
          Ok(_) => {},
          Err(_) => panic!("failed to lock reentrant mutex"),
        }
    });

    threads.push(t);
  }

  for t in threads {
      t.join().unwrap();
  }

  assert_eq!(*data.lock().unwrap(), 5 + 2 * NTHREADS * ITERATIONS);
}

#[cfg(any(target_arch = "aarch64",
          target_arch = "arm",
          target_arch = "powerpc",
          target_arch = "x86",
          target_arch = "x86_64"))]
fn main() {
  test_mutex();
  test_reentrant_mutex();
}
