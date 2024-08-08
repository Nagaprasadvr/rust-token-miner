use log::info;
use miner::{reduce_string, TokenAccount};
use std::sync::{atomic::AtomicBool, mpsc, Arc};
mod miner;

fn main() {
    env_logger::builder()
        .format_module_path(false)
        .format_timestamp_micros()
        .init();

    let difficulty = 10000000;
    let miner_threads_len = 10;

    let mut token_account = TokenAccount::create("Alice".to_string());

    loop {
        let (tx, rx) = mpsc::channel();
        let start = std::time::Instant::now();
        let (gen_nonce_hash, _gen_nonce) = miner::get_nonce_hash_with_nonce(difficulty);

        info!("challenge: {}", reduce_string(&gen_nonce_hash));

        let stop_signal = Arc::new(AtomicBool::new(false));
        let miner_threads = miner::mine(
            difficulty,
            miner_threads_len,
            gen_nonce_hash,
            tx,
            stop_signal.clone(),
        );

        while let Ok(data) = rx.recv() {
            info!("successfully mined challenge, nonce = {:?}", data.nonce);
            let elapsed = start.elapsed();
            info!("Time elapsed: {:?}", elapsed);

            token_account.add_tokens(elapsed.as_millis() as u64 * 10);
            stop_signal.store(true, std::sync::atomic::Ordering::Relaxed);
            break;
        }

        for handle in miner_threads {
            handle.join().unwrap();
        }

        info!(
            "{}'s tokens: {}",
            token_account.pubkey,
            token_account.get_tokens()
        );
        std::thread::sleep(std::time::Duration::from_secs(2));

        info!("New Mining round...");
    }
}
