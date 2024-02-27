#![no_main]

use libfuzzer_sys::fuzz_target;
use num_bigint::BigUint;
use summa_solvency::merkle_sum_tree::utils::{big_intify_username, big_uint_to_fp, fp_to_big_uint};

fuzz_target!(|data: &[u8]| {
    if let Ok(username) = std::str::from_utf8(data) {
        let big_uint_username = big_intify_username(username);

        let bytes_back = big_uint_username.to_bytes_be();
        assert_eq!(username.as_bytes(), bytes_back.as_slice());

        let fp_username = big_uint_to_fp(&big_uint_username);
        let big_uint_back = fp_to_big_uint(fp_username);

        let fp_back = big_uint_to_fp(&big_uint_back);
        assert_eq!(
            fp_username, fp_back,
            "Fp conversion cycle should be consistent"
        );
    }
});
