// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use rand::{self, Rng, SeedableRng, XorShiftRng};
use std::cell::RefCell;
use std::fmt::{self, Debug, Display, Formatter};
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

lazy_static! {
    static ref SEED: Mutex<Option<[u32; 4]>> = Mutex::new(None);
    static ref ALREADY_PRINTED: AtomicBool = AtomicBool::new(false);
    static ref GLOBAL_RNG: Mutex<Option<SeededRng>> = Mutex::new(None);
    static ref THREAD_SEED_OFFSET: Mutex<u32> = Mutex::new(1);
}

/// A [fast pseudorandom number generator]
/// (https://doc.rust-lang.org/rand/rand/struct.XorShiftRng.html) for use in tests which allows
/// seeding and prints the seed when the thread in which it is created panics.
pub struct SeededRng(XorShiftRng);

pub struct ThreadSeededRng(Rc<RefCell<SeededRng>>);

impl SeededRng {
    /// Construct a new `SeededRng` using a seed generated from cryptographically secure random
    /// data.
    ///
    /// The seed is only set once for the whole process, so every call to this will yield internal
    /// RNGs which are all seeded identically.
    pub fn new() -> Self {
        let optional_seed = &mut *unwrap!(SEED.lock());
        let seed = if let Some(current_seed) = *optional_seed {
            current_seed
        } else {
            let new_seed = [rand::random(), rand::random(), rand::random(), rand::random()];
            *optional_seed = Some(new_seed);
            new_seed
        };
        SeededRng(XorShiftRng::from_seed(seed))
    }

    /// Construct a new `SeededRng` using `seed`.
    ///
    /// If the underlying static seed has already been initialised to a value different to `seed`,
    /// then this function will panic.
    pub fn from_seed(seed: [u32; 4]) -> Self {
        let optional_seed = &mut *unwrap!(SEED.lock());
        if let Some(current_seed) = *optional_seed {
            if current_seed != seed {
                panic!("\nThe static seed has already been initialised to a different value via \
                        a call to `SeededRng::new()`\nor `SeededRng::from_seed(...)`.  This \
                        could be due to setting a hard-coded value for the seed in a\nsingle \
                        test case, but running the whole test suite.  If so, try running just \
                        the single test case.\n");
            }
        } else {
            *optional_seed = Some(seed);
        }

        SeededRng(XorShiftRng::from_seed(seed))
    }

    /// Constructs a thread-local `SeededRng`. The seed is generated via a global `SeededRng` using
    /// the global seed.
    pub fn thread_rng() -> ThreadSeededRng {
        thread_local!{
            static THREAD_SEEDED: Rc<RefCell<SeededRng>> = {
                let optional_rng = &mut *unwrap!(GLOBAL_RNG.lock());
                let rng = if let Some(ref mut rng) = *optional_rng {
                    rng
                } else {
                    *optional_rng = Some(SeededRng::new());
                    optional_rng.as_mut().unwrap()
                };
                let seed_offset = &mut *unwrap!(THREAD_SEED_OFFSET.lock());
                let seed = [rng.gen::<u32>().wrapping_add(*seed_offset),
                            rng.gen::<u32>().wrapping_add(*seed_offset),
                            rng.gen::<u32>().wrapping_add(*seed_offset),
                            rng.gen::<u32>().wrapping_add(*seed_offset)];
                *seed_offset += 1;
                Rc::new(RefCell::new(SeededRng(XorShiftRng::from_seed(seed))))
            }
        }
        ThreadSeededRng(THREAD_SEEDED.with(|x| x.clone()))
    }

    /// Construct a new `SeededRng`
    /// using a seed generated from random data provided by `self`.
    pub fn new_rng(&mut self) -> SeededRng {
        let new_seed = [self.0.gen(), self.0.gen(), self.0.gen(), self.0.gen()];
        SeededRng(XorShiftRng::from_seed(new_seed))
    }
}

impl Default for SeededRng {
    fn default() -> Self {
        SeededRng::new()
    }
}

impl Display for SeededRng {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter,
               "RNG seed: {:?}",
               *SEED.lock().unwrap_or_else(|e| e.into_inner()))
    }
}

impl Debug for SeededRng {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        <Self as Display>::fmt(self, formatter)
    }
}

impl Drop for SeededRng {
    fn drop(&mut self) {
        if thread::panicking() && !ALREADY_PRINTED.compare_and_swap(false, true, Ordering::SeqCst) {
            let msg = format!("{:?}", *SEED.lock().unwrap_or_else(|e| e.into_inner()));
            let border = (0..msg.len()).map(|_| "=").collect::<String>();
            println!("\n{}\n{}\n{}\n", border, msg, border);
        }
    }
}

impl Rng for SeededRng {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn choose<'a, T>(&mut self, arg: &'a [T]) -> Option<&'a T> {
        if arg.is_empty() {
            None
        } else {
            let index = self.gen_range(0, arg.len() as u32) as usize;
            Some(&arg[index])
        }
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        let mut i = values.len();
        while i >= 2 {
            i -= 1;
            values.swap(i, self.gen_range(0, (i + 1) as u32) as usize);
        }
    }
}

impl Rng for ThreadSeededRng {
    fn next_u32(&mut self) -> u32 {
        self.0.borrow_mut().next_u32()
    }

    fn choose<'a, T>(&mut self, arg: &'a [T]) -> Option<&'a T> {
        self.0.borrow_mut().choose(arg)
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        self.0.borrow_mut().shuffle(values)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use std::sync::atomic::Ordering;

    // We need the expected message here to ensure that any assertion failure in the test causes the
    // test to fail.  Only the final statement should cause a panic (calling `from_seed()` with a
    // different seed value).  This check can't be moved to its own test case since if it runs
    // first it will poison the mutex protecting the static seed, causing this test to fail.
    #[test]
    #[should_panic(expected = "\nThe static seed has already been initialised to a different value \
                               via a call to `SeededRng::new()`\nor `SeededRng::from_seed(...)`.  \
                               This could be due to setting a hard-coded value for the seed in \
                               a\nsingle test case, but running the whole test suite.  If so, try \
                               running just the single test case.\n")]
    fn seeded_rng() {
        {
            let seed = [0, 1, 2, 3];
            let mut seeded_rng1 = SeededRng::from_seed(seed);
            let mut seeded_rng2 = SeededRng::new();
            let expected = 12884903946;
            assert_eq!(seeded_rng1.next_u64(), expected);
            assert_eq!(seeded_rng2.next_u64(), expected);

            let mut rng1_from_seeded_rng1 = seeded_rng1.new_rng();
            let mut rng2_from_seeded_rng1 = seeded_rng1.new_rng();
            let expected1 = 36055743652167817;
            let expected2 = 19781043125127688;
            assert_eq!(rng1_from_seeded_rng1.next_u64(), expected1);
            assert_eq!(rng2_from_seeded_rng1.next_u64(), expected2);

            let mut rng1_from_seeded_rng2 = seeded_rng2.new_rng();
            let mut rng2_from_seeded_rng2 = seeded_rng2.new_rng();
            assert_eq!(rng1_from_seeded_rng2.next_u64(), expected1);
            assert_eq!(rng2_from_seeded_rng2.next_u64(), expected2);
        }

        let _ = SeededRng::from_seed([3, 2, 1, 0]);
    }

    // Run this in isolation to `seeded_rng` test as it assumes `ALREADY_PRINTED` is not hampered
    // by other tests (`seeded_rng` test will interfere with this assumption and will produce
    // random failures as `ALREADY_PRINTED` is a global)
    //
    // NOTE:
    // Do not change the name of this function without changing it in the CI scripts.
    #[ignore]
    #[test]
    fn print_seed_only_once_for_multiple_failures() {
        assert!(!ALREADY_PRINTED.load(Ordering::Relaxed));
        let _ = SeededRng::new();
        assert!(!ALREADY_PRINTED.load(Ordering::Relaxed));

        for _ in 0..2 {
            let j = thread::spawn(move || {
                                      let _rng = SeededRng::new();
                                      panic!("This is an induced panic to test if \
                                             `ALREADY_PRINTED` global is toggled only once on \
                                             panic");
                                  });

            assert!(j.join().is_err());
            assert!(ALREADY_PRINTED.load(Ordering::Relaxed));
        }
    }
}
