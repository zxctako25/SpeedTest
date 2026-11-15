use super::model::{
    ConnectionInfo, Phase, TestEvent
};

use eframe::egui;
use std::collections::VecDeque;
use std::io::{
    self, Read
};

use std::sync::atomic::{
    AtomicBool, AtomicU64, Ordering
};

use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use std::time::{
    Duration, Instant
};

const BASE: &str = "https://speed.cloudflare.com";
const THREADS: usize = 2;
const PING_COUNT: usize = 10;
const TEST_SECS: f64 = 8.0;
const WARMUP_SECS: f64 = 1.5;
const DOWNLOAD_CHUNK: u64 = 50_000_000;
const UPLOAD_CHUNK: usize = 2_000_000;

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Download,
    Upload,
}

struct Emitter {
    tx: Sender<TestEvent>,
    ctx: egui::Context,
}

impl Emitter {
    fn send(&self, ev: TestEvent) {
        let _ = self.tx.send(ev);
        self.ctx.request_repaint();
    }
}

pub struct SpeedTestEngine {
    cancel: Arc<AtomicBool>,
}

impl SpeedTestEngine {
    pub fn start(tx: Sender<TestEvent>, ctx: egui::Context) -> Self {
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        thread::spawn(move || {
            let emit = Emitter { tx, ctx };
            match run(&emit, &flag) {
                Ok(()) => emit.send(TestEvent::Finished),
                Err(e) => emit.send(TestEvent::Failed(e)),
            }
        });
        Self { cancel }
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

impl Drop for SpeedTestEngine {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn run(emit: &Emitter, cancel: &Arc<AtomicBool>) -> Result<(), String> {
    let agent = build_agent();

    emit.send(TestEvent::PhaseChanged(Phase::Ping));
    if let Some(info) = fetch_connection_info(&agent) {
        emit.send(TestEvent::Connection(info));
    }

    let (ping_ms, jitter_ms) = measure_ping(&agent, cancel)?;
    emit.send(TestEvent::PingDone { ping_ms, jitter_ms });

    emit.send(TestEvent::PhaseChanged(Phase::Download));
    let download = measure_throughput(&agent, Direction::Download, emit, cancel)?;
    emit.send(TestEvent::DownloadDone(download));

    emit.send(TestEvent::PhaseChanged(Phase::Upload));
    let upload = measure_throughput(&agent, Direction::Upload, emit, cancel)?;
    emit.send(TestEvent::UploadDone(upload));

    Ok(())
}

pub fn build_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(10))
        .timeout_write(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
}

pub fn fetch_connection_info(agent: &ureq::Agent) -> Option<ConnectionInfo> {
    let mut ip = String::new();
    let mut isp = String::new();
    let mut city = String::new();
    let mut country = String::new();

    // IP всегда берём с ip-api.com (IPv4)
    if let Ok(resp) = agent
        .get("http://ip-api.com/json/?fields=status,country,countryCode,city,isp,query")
        .call()
    {
        if let Ok(body) = resp.into_string() {
            let get = |key: &str| -> Option<String> {
                let pat = format!("\"{}\":\"", key);
                body.find(&pat).and_then(|pos| {
                    let start = pos + pat.len();
                    body[start..].find('"').map(|end| body[start..start + end].to_string())
                })
            };
            
            if get("status").as_deref() == Some("success") {
                ip = get("query").unwrap_or_default();
                isp = get("isp").unwrap_or_default();
                city = get("city").unwrap_or_default();
                country = get("country").unwrap_or_default();
            }
        }
    }

    if ip.is_empty() {
        return None;
    }

    Some(ConnectionInfo {
        ip,
        isp,
        city,
        country,
        server: String::new(),
    })
}

fn measure_ping(agent: &ureq::Agent, cancel: &AtomicBool) -> Result<(f64, f64), String> {
    let url = format!("{BASE}/__down?bytes=0");
    let mut samples: Vec<f64> = Vec::new();

    for i in 0..=PING_COUNT {
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled".to_string());
        }
        
        let t = Instant::now();
        let resp = agent
            .get(&url)
            .set("Accept", "*/*")
            .set("Origin", "https://speed.cloudflare.com")
            .call()
            .map_err(|_| "Could not reach the test server. Check your connection.".to_string())?;
        let mut r = resp.into_reader();
        let _ = io::copy(&mut r, &mut io::sink());
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if i > 0 {
            samples.push(ms);
        }
        thread::sleep(Duration::from_millis(50));
    }

    let mut sorted = samples.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted[sorted.len() / 2];
    let jitter = samples.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>()
        / (samples.len().saturating_sub(1).max(1)) as f64;

    Ok((median, jitter))
}

fn mbps(bytes: u64, secs: f64) -> f64 {
    bytes as f64 * 8.0 / secs / 1_000_000.0
}

fn measure_throughput(
    agent: &ureq::Agent,
    dir: Direction,
    emit: &Emitter,
    cancel: &Arc<AtomicBool>,
) -> Result<f64, String> {
    let bytes = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let failed = Arc::new(AtomicBool::new(false));

    for _ in 0..THREADS {
        let (agent, bytes, stop, failed) =
            (agent.clone(), bytes.clone(), stop.clone(), failed.clone());
        
        thread::spawn(move || match dir {
            Direction::Download => download_worker(agent, bytes, stop, failed),
            Direction::Upload => upload_worker(agent, bytes, stop, failed),
        });
    }

    let start = Instant::now();
    let mut window: VecDeque<(Instant, u64)> = VecDeque::new();
    let mut warm: Option<(Instant, u64)> = None;

    let result = loop {
        thread::sleep(Duration::from_millis(100));
        if cancel.load(Ordering::Relaxed) {
            break Err("Cancelled".to_string());
        }
        if failed.load(Ordering::Relaxed) {
            break Err("The test server rejected the request. (Попробуйте еще раз)".to_string());
        }

        let now = Instant::now();
        let total = bytes.load(Ordering::Relaxed);
        let elapsed = now.duration_since(start).as_secs_f64();
        if total == 0 && elapsed > 5.0 {
            break Err("No data received. Check your connection.".to_string());
        }

        window.push_back((now, total));
        while let Some(&(t, _)) = window.front() {
            if window.len() > 2 && now.duration_since(t).as_secs_f64() > 1.0 {
                window.pop_front();
            } else {
                break;
            }
        }
        
        if let (Some(&(t0, b0)), Some(&(t1, b1))) = (window.front(), window.back()) {
            let dt = t1.duration_since(t0).as_secs_f64();
            if dt > 0.0 {
                emit.send(TestEvent::LiveSpeed(mbps(b1 - b0, dt)));
            }
        }

        if warm.is_none() && elapsed >= WARMUP_SECS {
            warm = Some((now, total));
        }

        if elapsed >= TEST_SECS {
            let (t0, b0) = warm.unwrap_or((start, 0));
            let dt = now.duration_since(t0).as_secs_f64();
            break Ok(mbps(total - b0, dt));
        }
    };

    stop.store(true, Ordering::Relaxed);
    result
}

fn download_worker(
    agent: ureq::Agent,
    bytes: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    failed: Arc<AtomicBool>,
) {
    let url = format!("{BASE}/__down?bytes={DOWNLOAD_CHUNK}");
    let mut buf = vec![0u8; 64 * 1024];

    while !stop.load(Ordering::Relaxed) {
        let resp = match agent.get(&url)
            .set("Accept", "*/*")
            .set("Origin", "https://speed.cloudflare.com")
            .call() {
            Ok(r) => r,
            Err(ureq::Error::Status(code, _)) => {
                if !stop.load(Ordering::Relaxed) {
                    eprintln!("Download error status: {}", code);
                    failed.store(true, Ordering::Relaxed);
                }
                return;
            }
            Err(e) => {
                if !stop.load(Ordering::Relaxed) {
                    eprintln!("Download error: {}", e);
                }
                thread::sleep(Duration::from_millis(200));
                continue;
            }
        };

        let mut reader = resp.into_reader();
        loop {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    bytes.fetch_add(n as u64, Ordering::Relaxed);
                }
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
}

struct UploadBody {
    remaining: usize,
    bytes: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
}

impl Read for UploadBody {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.stop.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::Other, "stopped"));
        }
        if self.remaining == 0 {
            return Ok(0);
        }
        
        let n = buf.len().min(self.remaining).min(32 * 1024);
        buf[..n].fill(0);
        self.remaining -= n;
        self.bytes.fetch_add(n as u64, Ordering::Relaxed);
        Ok(n)
    }
}

fn upload_worker(
    agent: ureq::Agent,
    bytes: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    failed: Arc<AtomicBool>,
) {
    let url = format!("{BASE}/__up");

    while !stop.load(Ordering::Relaxed) {
        let body = UploadBody {
            remaining: UPLOAD_CHUNK,
            bytes: bytes.clone(),
            stop: stop.clone(),
        };
        let res = agent
            .post(&url)
            .set("Content-Type", "application/octet-stream")
            .set("Content-Length", &UPLOAD_CHUNK.to_string())
            .set("Origin", "https://speed.cloudflare.com")
            .send(body);

        match res {
            Err(ureq::Error::Status(code, _)) => {
                if !stop.load(Ordering::Relaxed) {
                    eprintln!("Upload error status: {}", code);
                    failed.store(true, Ordering::Relaxed);
                }
                return;
            }
            Err(e) => {
                if !stop.load(Ordering::Relaxed) {
                    eprintln!("Upload error: {}", e);
                    thread::sleep(Duration::from_millis(200));
                }
            }
            Ok(_) => {}
        }
        thread::sleep(Duration::from_millis(50));
    }
}
