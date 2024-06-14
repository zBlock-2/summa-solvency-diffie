#![no_main]
use libfuzzer_sys::fuzz_target;
use rand::{distributions::Alphanumeric, Rng};
use std::io::Write;
use summa_solvency::merkle_sum_tree::utils::csv_parser::parse_csv_to_entries;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    writeln!(temp_file, "username,balance_ETH_ETH,balance_USDT_ETH")
        .expect("Failed to write header to temporary file");

    let mut rng = rand::thread_rng();
    for _ in 0..data.len() {
        let username: String = (&mut rng)
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        let balance_eth: String = rng.gen_range(1..=10_000_000).to_string();
        let balance_usdt: String = rng.gen_range(1..=10_000_000).to_string();

        writeln!(temp_file, "{},{},{}", username, balance_eth, balance_usdt)
            .expect("Failed to write data to temporary file");
    }

    temp_file.flush().expect("Failed to flush temporary file");

    let result = parse_csv_to_entries::<_, 2, 8>(temp_file.path());

    match result {
        Ok((cryptocurrencies, entries)) => {
            assert!(cryptocurrencies.len() <= 2);
            for entry in entries {
                assert!(entry.balances().len() == 2);
            }
        }
        Err(e) => {
            eprintln!("Error parsing CSV: {}", e);
        }
    }
});
