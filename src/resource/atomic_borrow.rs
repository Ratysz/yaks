// Following file contains modified copy of a fragment of `hecs` library source code.
// Original license note is reproduced in the next comment block.

// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::atomic::{AtomicUsize, Ordering};

pub struct AtomicBorrow(AtomicUsize);

impl AtomicBorrow {
    const UNIQUE_BIT: usize = !(usize::max_value() >> 1);

    pub const fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    pub fn is_free(&self) -> bool {
        self.0.load(Ordering::Acquire) == 0
    }

    pub fn borrow(&self) -> bool {
        let value = self.0.fetch_add(1, Ordering::Acquire).wrapping_add(1);
        if value == 0 {
            // Wrapped, this borrow is invalid!
            core::panic!()
        }
        if value & AtomicBorrow::UNIQUE_BIT != 0 {
            self.0.fetch_sub(1, Ordering::Release);
            false
        } else {
            true
        }
    }

    pub fn borrow_mut(&self) -> bool {
        self.0
            .compare_exchange(
                0,
                AtomicBorrow::UNIQUE_BIT,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_ok()
    }

    pub fn release(&self) {
        let value = self.0.fetch_sub(1, Ordering::Release);
        debug_assert!(value != 0, "unbalanced release");
        debug_assert!(
            value & AtomicBorrow::UNIQUE_BIT == 0,
            "shared release of unique borrow"
        );
    }

    pub fn release_mut(&self) {
        self.0.store(0, Ordering::Release);
    }
}
