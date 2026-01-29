mod wallet;

use wallet::QuantumWallet;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Write, Read};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use warp::Filter;
use std::collections::HashMap;
use std::env;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SignedPulse {
    state: PersistentState,
    signature: Vec<u8>,
    public_key: Vec<u8>,
    raw_json: String, 
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PersistentState {
    neural_fund: f64,
    balances: HashMap<String, f64>,
    #[serde(default)]
    node_scores: HashMap<String, f64>,
    global_brain: Vec<f64>,
    intelligence_level: f64,
    #[serde(default)]
    difficulty: f64,
}

struct HimeraNode {
    state: PersistentState,
    filename: String,
    miner_addr: String,
    wallet: QuantumWallet,
}

impl HimeraNode {
    fn new(wallet: QuantumWallet, filename: &str, miner_addr: &str) -> Self {
        if let Ok(mut file) = File::open(filename) {
            let mut data = Vec::new();
            if file.read_to_end(&mut data).is_ok() {
                if let Ok(dec) = decompress_size_prepended(&data) {
                    if let Ok(s) = serde_json::from_slice(&dec) {
                        return HimeraNode { state: s, filename: filename.into(), miner_addr: miner_addr.into(), wallet };
                    }
                }
            }
        }
        HimeraNode {
            state: PersistentState { 
                neural_fund: 0.0, 
                balances: HashMap::from([(wallet.address.clone(), 1000.0)]),
                node_scores: HashMap::new(),
                global_brain: vec![0.5; 10],
                intelligence_level: 0.0001,
                difficulty: 1.0,
            },
            filename: filename.into(),
            miner_addr: miner_addr.into(),
            wallet,
        }
    }

    fn update_score(&mut self, is_valid: bool) {
        let current = self.state.node_scores.entry("network_peer".to_string()).or_insert(1.0);
        if is_valid { *current = (*current + 0.01).min(2.0); } 
        else { *current = (*current - 0.5).max(0.0); }
    }

    fn verify_incoming_work(&self, incoming: &PersistentState) -> bool {
        let delta_intel = incoming.intelligence_level - self.state.intelligence_level;
        if delta_intel > 0.05 { return false; } 
        let mut shift = 0.0;
        for (i, w) in incoming.global_brain.iter().enumerate() {
            shift += (w - self.state.global_brain[i]).abs();
        }
        if delta_intel > 0.0 && shift == 0.0 { return false; }
        true
    }

    fn apply_learning_pulse(&mut self) {
        self.state.difficulty = 1.0 + (self.state.intelligence_level * 15.0);
        let lr = 0.05 / self.state.difficulty; 
        let target = (chrono::Utc::now().timestamp() % 10) as f64 / 10.0;
        for val in self.state.global_brain.iter_mut() {
            *val += (target - *val) * lr;
        }
        let m_bal = self.state.balances.entry(self.miner_addr.clone()).or_insert(0.0);
        let reward = 1.0; 
        let tax = reward * 0.0015; 
        *m_bal += reward - tax;
        self.state.neural_fund += tax;
        self.state.intelligence_level += 0.0002 / self.state.difficulty;
        self.save();
    }

    fn save(&self) {
        if let Ok(ser) = serde_json::to_vec(&self.state) {
            let comp = compress_prepend_size(&ser);
            if let Ok(mut f) = File::create(&self.filename) { let _ = f.write_all(&comp); }
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let p2p = args.get(1).cloned().unwrap_or("8080".into());
    let api: u16 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3030);
    let db = args.get(3).cloned().unwrap_or("node.lz4".into());
    let wallet_f = args.get(4).cloned().unwrap_or("wallet.him".into());
    let miner = args.get(5).cloned().unwrap_or("HIM_MINER_BEE".into());

    let wallet = QuantumWallet::new(&wallet_f);
    let node = Arc::new(Mutex::new(HimeraNode::new(wallet, &db, &miner)));

    println!("🚀 HIMERA HIM v8.3 | PORT: {} | API: {}", p2p, api);

    let n_sync = node.clone();
    let my_p = p2p.clone();
    tokio::spawn(async move {
        let targets = vec!["8080", "8081", "8082"];
        loop {
            tokio::time::sleep(Duration::from_millis(3000)).await;
            let pulse = {
                let mut n = n_sync.lock().await;
                n.apply_learning_pulse(); 
                let json = serde_json::to_string(&n.state).unwrap();
                let sig = n.wallet.sign_message(json.as_bytes());
                SignedPulse { state: n.state.clone(), signature: sig, public_key: n.wallet.public_key.to_vec(), raw_json: json }
            };
            if let Ok(data) = serde_json::to_vec(&pulse) {
                for port in &targets {
                    if *port == my_p { continue; }
                    if let Ok(mut s) = TcpStream::connect(format!("127.0.0.1:{}", port)).await { let _ = s.write_all(&data).await; }
                }
            }
        }
    });

    let n_api = node.clone();
    tokio::spawn(warp::serve(warp::path("state").then(move || {
        let n = n_api.clone();
        async move { warp::reply::json(&n.lock().await.state) }
    })).run(([0, 0, 0, 0], api)));

    let listener = TcpListener::bind(format!("0.0.0.0:{}", p2p)).await.unwrap();
    loop {
        if let Ok((mut s, _)) = listener.accept().await {
            let n_i = node.clone();
            tokio::spawn(async move {
                let mut buf = vec![0; 256 * 1024]; 
                if let Ok(len) = s.read(&mut buf).await {
                    if let Ok(pulse) = serde_json::from_slice::<SignedPulse>(&buf[..len]) {
                        let valid_sig = QuantumWallet::verify_signature(pulse.raw_json.as_bytes(), &pulse.signature, &pulse.public_key);
                        let mut n = n_i.lock().await;
                        if valid_sig && n.verify_incoming_work(&pulse.state) {
                            if pulse.state.intelligence_level > n.state.intelligence_level {
                                n.state = pulse.state; n.save();
                                println!("✅ [AUDIT OK] Intel: {:.4}", n.state.intelligence_level);
                            }
                            n.update_score(true);
                        } else { n.update_score(false); }
                    }
                }
            });
        }
    }
}
