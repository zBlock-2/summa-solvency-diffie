#![no_main]

use libfuzzer_sys::fuzz_target;
use summa_backend::merkle_sum_tree::utils::big_intify_username;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    if let Ok(s) = std::str::from_utf8(data){
        let _ = big_intify_username(s);
    };
    
});
