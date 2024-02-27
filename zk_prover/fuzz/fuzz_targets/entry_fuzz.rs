#![no_main]
use libfuzzer_sys::fuzz_target;
use num_bigint::BigUint;
use rand::{distributions::Alphanumeric, Rng};
use std::str;
use summa_solvency::merkle_sum_tree::utils::operation_helpers::big_intify_username;
use summa_solvency::merkle_sum_tree::utils::operation_helpers::fp_to_big_uint;
use summa_solvency::merkle_sum_tree::{Entry, Node};

fuzz_target!(|data: &[u8]| {
    let username = str::from_utf8(&data).unwrap_or("default_user");
    let mut rng = rand::thread_rng();
    let balances: [BigUint; 2] = [
        BigUint::from(rng.gen_range(0..10_000_000) as u64),
        BigUint::from(rng.gen_range(0..10_000_000) as u64),
    ];

    // Attempt to create an Entry
    match Entry::new(username.to_string(), balances.clone()) {
        Ok(mut entry) => {
            assert_eq!(entry.username_as_big_uint(), &big_intify_username(username));

            assert_eq!(&entry.balances, &balances);

            let leaf = entry.compute_leaf();
            let x = fp_to_big_uint(leaf.balances[0]);
            let y = fp_to_big_uint(leaf.balances[1]);
            let z: [BigUint; 2] = [x, y];

            assert_eq!(&z, &balances);

            let updated_balances: [BigUint; 2] = [
                BigUint::from(rng.gen_range(1000..2000) as u64),
                BigUint::from(rng.gen_range(1000..2000) as u64),
            ];

            let updated_leaf = entry.recompute_leaf(&updated_balances);
            let u = fp_to_big_uint(updated_leaf.balances[0]);
            let v = fp_to_big_uint(updated_leaf.balances[1]);
            let w: [BigUint; 2] = [u, v];

            assert_eq!(&w, &updated_balances);
        }

        Err(e) => {
            eprintln!("Failed to create Entry: {}", e);
        }
    }
});
