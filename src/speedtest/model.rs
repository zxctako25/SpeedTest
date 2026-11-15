#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Phase {
    Idle,
    Ping,
    Download,
    Upload,
    Done,
    Failed,
}

impl Phase {
    pub fn caption(self) -> &'static str {
        match self {
            Phase::Idle => "READY",
            Phase::Ping => "PING",
            Phase::Download => "DOWNLOAD",
            Phase::Upload => "UPLOAD",
            Phase::Done => "COMPLETE",
            Phase::Failed => "FAILED",
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct TestResult {
    pub ping_ms: Option<f64>,
    pub jitter_ms: Option<f64>,
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
}

#[derive(Clone, Default, Debug)]
pub struct ConnectionInfo {
    pub ip: String,
    pub isp: String,
    pub city: String,
    pub country: String,
    pub server: String,
}

#[derive(Debug)]
pub enum TestEvent {
    PhaseChanged(Phase),
    Connection(ConnectionInfo),
    PingDone { ping_ms: f64, jitter_ms: f64 },
    LiveSpeed(f64),
    DownloadDone(f64),
    UploadDone(f64),
    Failed(String),
    Finished,
}
