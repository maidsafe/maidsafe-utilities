// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use rand::{self, Rng, SeedableRng, XorShiftRng};

lazy_static! {
    static ref IS_INITIALISED: AtomicBool = AtomicBool::new(false);
    static ref SEED: Mutex<[u32; 4]> =
        Mutex::new([rand::random(), rand::random(), rand::random(), rand::random()]);
}

/// Error indicating that the static seed has already been initialised to a different value.
#[derive(Debug)]
pub struct AlreadySeeded;

impl Display for AlreadySeeded {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter,
               "The static seed has already been initialised to a different value via a call to \
                `SeededRng::new()` or `SeededRng::from_seed()`.")
    }
}

impl Error for AlreadySeeded {
    fn description(&self) -> &str {
        "already seeded"
    }
}

/// A [fast pseudorandom number generator]
/// (https://doc.rust-lang.org/rand/rand/struct.XorShiftRng.html) that allows seeding and prints the
/// seed when the thread in which it is created panics.
pub struct SeededRng {
    seed: [u32; 4],
    inner: XorShiftRng,
}

impl SeededRng {
    /// Construct a new `SeededRng` using a seed generated from cryptographically secure random
    /// data.  The seed is only set once for the whole process, so every call to this will yield
    /// internal RNGs which are all seeded identically.
    pub fn new() -> Self {
        let seed: [u32; 4] = *unwrap!(SEED.lock());
        IS_INITIALISED.store(true, Ordering::Relaxed);
        SeededRng {
            seed: seed,
            inner: XorShiftRng::from_seed(seed),
        }
    }

    /// Construct a new `SeededRng` using `seed`.
    pub fn from_seed(seed: [u32; 4]) -> Result<Self, AlreadySeeded> {
        let mut current_seed = &mut *unwrap!(SEED.lock());
        if IS_INITIALISED.load(Ordering::Relaxed) {
            if *current_seed != seed {
                return Err(AlreadySeeded);
            }
        } else {
            *current_seed = seed;
            IS_INITIALISED.store(true, Ordering::Relaxed);
        }

        Ok(SeededRng {
            seed: *current_seed,
            inner: XorShiftRng::from_seed(*current_seed),
        })
    }

    /// Construct a new [`XorShiftRng`](https://doc.rust-lang.org/rand/rand/struct.XorShiftRng.html)
    /// using a seed generated from random data provided by `self`.
    pub fn new_rng(&mut self) -> XorShiftRng {
        XorShiftRng::from_seed([self.inner.gen(),
                                self.inner.gen(),
                                self.inner.gen(),
                                self.inner.gen()])
    }
}

impl Default for SeededRng {
    fn default() -> Self {
        SeededRng::new()
    }
}

impl Display for SeededRng {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "RNG seed: {:?}", self.seed)
    }
}

impl Debug for SeededRng {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        <Self as Display>::fmt(self, formatter)
    }
}

impl Drop for SeededRng {
    fn drop(&mut self) {
        if thread::panicking() {
            let msg = format!("{}", self);
            let border = (0..msg.len()).map(|_| "=").collect::<String>();
            println!("\n{}\n{}\n{}\n", border, msg, border);
        }
    }
}

impl Rng for SeededRng {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn seeded_rng() {
        let seed = [0, 1, 2, 3];
        let mut seeded_rng1 = unwrap!(SeededRng::from_seed(seed));
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

        assert!(SeededRng::from_seed([3, 2, 1, 0]).is_err());
    }
}
