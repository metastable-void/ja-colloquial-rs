#[cfg(not(target_has_atomic = "32"))]
compile_error!("ja-colloquial requires lock-free 32-bit atomics");

#[cfg(target_has_atomic = "32")]
mod supported {
    use core::hint::spin_loop;
    use core::sync::atomic::{AtomicU32, Ordering};

    static EPOCH: AtomicU32 = AtomicU32::new(0);
    static KEY: [AtomicU32; 4] = [const { AtomicU32::new(0) }; 4];
    static BLOCK_COUNTER: AtomicU32 = AtomicU32::new(0);

    const CONSTANTS: [u32; 4] = [0x6170_7865, 0x3120_646e, 0x7962_2d36, 0x6b20_6574];

    pub(crate) fn seed(seed: [u8; 16]) {
        let stable_epoch = loop {
            let epoch = EPOCH.load(Ordering::SeqCst);
            if epoch & 1 != 0 {
                spin_loop();
                continue;
            }
            assert!(
                epoch < u32::MAX - 1,
                "ja-colloquial global RNG reseed epoch exhausted"
            );
            if EPOCH
                .compare_exchange(epoch, epoch + 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                break epoch;
            }
        };

        for (index, word) in KEY.iter().enumerate() {
            let offset = index * 4;
            word.store(
                u32::from_le_bytes([
                    seed[offset],
                    seed[offset + 1],
                    seed[offset + 2],
                    seed[offset + 3],
                ]),
                Ordering::SeqCst,
            );
        }
        BLOCK_COUNTER.store(0, Ordering::SeqCst);
        EPOCH.store(stable_epoch + 2, Ordering::SeqCst);
    }

    fn reserve_block() -> ([u32; 4], u32) {
        loop {
            let epoch = EPOCH.load(Ordering::SeqCst);
            if epoch & 1 != 0 {
                spin_loop();
                continue;
            }

            let key = [
                KEY[0].load(Ordering::SeqCst),
                KEY[1].load(Ordering::SeqCst),
                KEY[2].load(Ordering::SeqCst),
                KEY[3].load(Ordering::SeqCst),
            ];
            let reservation =
                BLOCK_COUNTER.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                    current.checked_add(1)
                });
            let final_epoch = EPOCH.load(Ordering::SeqCst);

            match reservation {
                Ok(counter) if final_epoch == epoch => return (key, counter),
                Ok(_) => continue,
                Err(_) if final_epoch != epoch => continue,
                Err(_) => panic!("ja-colloquial global RNG block counter exhausted"),
            }
        }
    }

    fn next_u64() -> u64 {
        let (key, counter) = reserve_block();
        let block = chacha20_block(key, counter);
        u64::from_le_bytes([
            block[0], block[1], block[2], block[3], block[4], block[5], block[6], block[7],
        ])
    }

    pub(crate) fn random_index(upper: usize) -> usize {
        random_index_with(upper, next_u64)
    }

    fn random_index_with(upper: usize, mut next: impl FnMut() -> u64) -> usize {
        assert!(upper != 0, "random index upper bound must be nonzero");
        let bound = u64::try_from(upper).expect("random index upper bound exceeds u64");
        let rejection_threshold = bound.wrapping_neg() % bound;
        loop {
            let value = next();
            if value >= rejection_threshold {
                return (value % bound) as usize;
            }
        }
    }

    fn chacha20_block(key: [u32; 4], counter: u32) -> [u8; 64] {
        let input = [
            CONSTANTS[0],
            CONSTANTS[1],
            CONSTANTS[2],
            CONSTANTS[3],
            key[0],
            key[1],
            key[2],
            key[3],
            key[0],
            key[1],
            key[2],
            key[3],
            counter,
            0,
            0,
            0,
        ];
        let mut state = input;

        for _ in 0..10 {
            quarter_round(&mut state, 0, 4, 8, 12);
            quarter_round(&mut state, 1, 5, 9, 13);
            quarter_round(&mut state, 2, 6, 10, 14);
            quarter_round(&mut state, 3, 7, 11, 15);
            quarter_round(&mut state, 0, 5, 10, 15);
            quarter_round(&mut state, 1, 6, 11, 12);
            quarter_round(&mut state, 2, 7, 8, 13);
            quarter_round(&mut state, 3, 4, 9, 14);
        }

        let mut output = [0_u8; 64];
        for (index, (word, original)) in state.iter().zip(input).enumerate() {
            let bytes = word.wrapping_add(original).to_le_bytes();
            output[index * 4..index * 4 + 4].copy_from_slice(&bytes);
        }
        output
    }

    fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
        state[a] = state[a].wrapping_add(state[b]);
        state[d] ^= state[a];
        state[d] = state[d].rotate_left(16);

        state[c] = state[c].wrapping_add(state[d]);
        state[b] ^= state[c];
        state[b] = state[b].rotate_left(12);

        state[a] = state[a].wrapping_add(state[b]);
        state[d] ^= state[a];
        state[d] = state[d].rotate_left(8);

        state[c] = state[c].wrapping_add(state[d]);
        state[b] ^= state[c];
        state[b] = state[b].rotate_left(7);
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::panic;
        use std::thread;
        use std::vec::Vec;

        #[test]
        fn original_128_bit_zero_vector() {
            let expected = [
                0x89, 0x67, 0x09, 0x52, 0x60, 0x83, 0x64, 0xfd, 0x00, 0xb2, 0xf9, 0x09, 0x36, 0xf0,
                0x31, 0xc8, 0xe7, 0x56, 0xe1, 0x5d, 0xba, 0x04, 0xb8, 0x49, 0x3d, 0x00, 0x42, 0x92,
                0x59, 0xb2, 0x0f, 0x46, 0xcc, 0x04, 0xf1, 0x11, 0x24, 0x6b, 0x6c, 0x2c, 0xe0, 0x66,
                0xbe, 0x3b, 0xfb, 0x32, 0xd9, 0xaa, 0x0f, 0xdd, 0xfb, 0xc1, 0x21, 0x23, 0xd4, 0xb9,
                0xe4, 0x4f, 0x34, 0xdc, 0xa0, 0x5a, 0x10, 0x3f,
            ];
            assert_eq!(chacha20_block([0; 4], 0), expected);
        }

        #[test]
        fn bounded_rejection_path() {
            let mut values = [5_u64, 17].into_iter();
            assert_eq!(random_index_with(10, || values.next().unwrap()), 7);
        }

        #[test]
        fn global_protocol() {
            seed([0x5a; 16]);
            let first: Vec<_> = (0..32).map(|_| random_index(31_086)).collect();
            seed([0x5a; 16]);
            let second: Vec<_> = (0..32).map(|_| random_index(31_086)).collect();
            assert_eq!(first, second);

            seed([1; 16]);
            let workers: Vec<_> = (0..8)
                .map(|_| thread::spawn(|| (0..128).map(|_| reserve_block().1).collect::<Vec<_>>()))
                .collect();
            let mut counters: Vec<_> = workers
                .into_iter()
                .flat_map(|worker| worker.join().unwrap())
                .collect();
            counters.sort_unstable();
            assert_eq!(counters, (0..1024).collect::<Vec<_>>());

            seed([2; 16]);
            BLOCK_COUNTER.store(u32::MAX, Ordering::SeqCst);
            assert!(panic::catch_unwind(next_u64).is_err());
            seed([0; 16]);
        }
    }
}

#[cfg(target_has_atomic = "32")]
pub(crate) use supported::{random_index, seed};
