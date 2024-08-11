use indicatif::{ProgressBar, ProgressStyle};
use rand::{self, Rng};
use sha2::{Digest, Sha512};

use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct MineResult {
    pub nonce: u64,
}

pub struct TokenAccount {
    pub pubkey: String,
    pub tokens: f32,
}

impl TokenAccount {
    pub fn create(name: String) -> Self {
        TokenAccount {
            pubkey: reduce_string(&bytes_to_base58(&hash(name))),
            tokens: 0.0,
        }
    }

    pub fn get_tokens(&self) -> f32 {
        return self.tokens;
    }

    pub fn add_tokens(&mut self, amount: f32) {
        self.tokens += amount;
    }
}

pub fn mine(
    difficulty: u64,
    miner_threads_len: usize,
    nonce_hash: String,
    tx: Sender<MineResult>,
    stop_signal: Arc<AtomicBool>,
) -> Vec<JoinHandle<()>> {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.blue} {msg}")
            .unwrap()
            .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"]),
    );
    pb.set_message("Mining...");

    let mut miner_threads: Vec<JoinHandle<()>> = Vec::with_capacity(miner_threads_len);

    let tx_clone = tx.clone();
    let nonce_hash_arc = Arc::new(nonce_hash);
    let pb_arc = Arc::new(pb);

    for _ in 0..miner_threads_len {
        let tx_clone = tx_clone.clone();
        let nonce_hash_clone = Arc::clone(&nonce_hash_arc);
        let stop_signal = Arc::clone(&stop_signal);
        let pb_clone = Arc::clone(&pb_arc);
        let handle: JoinHandle<()> = thread::spawn(move || loop {
            if stop_signal.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            let (calc_nonce_hash, calc_nonce) = get_nonce_hash_with_nonce(difficulty);
            if calc_nonce_hash.eq(nonce_hash_clone.as_ref()) {
                tx_clone.send(MineResult { nonce: calc_nonce }).unwrap();
                pb_clone.finish();
                break;
            }
        });

        miner_threads.push(handle)
    }

    return miner_threads;
}

pub fn get_nonce_hash_with_nonce(difficulty: u64) -> (String, u64) {
    let mut generator = rand::thread_rng();
    let calc_nonce = generator.gen_range(0..=difficulty);

    let mut hasher = Sha512::new();
    hasher.update(calc_nonce.to_string());
    let hash_result = hasher.finalize();

    let hex_string = format!("{:x}", hash_result);

    return (hex_string, calc_nonce);
}

pub fn hash(data: String) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.update(data);
    let hash_result = hasher.finalize();

    return hash_result.to_vec();
}

pub fn bytes_to_base58(data: &[u8]) -> String {
    bs58::encode(data).into_string()
}

pub fn reduce_string(data: &String) -> String {
    data[0..5].to_string() + "..." + &data[data.len() - 5..]
}
