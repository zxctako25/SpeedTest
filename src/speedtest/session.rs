use super::engine::{
    build_agent, fetch_connection_info, SpeedTestEngine
};

use super::model::{
    ConnectionInfo, Phase, TestEvent, TestResult
};

use eframe::egui;
use std::sync::mpsc::{
    self, Receiver
};

pub struct Session {
    pub phase: Phase,
    pub result: TestResult,
    pub live_mbps: f64,
    pub info: Option<ConnectionInfo>,
    pub error: Option<String>,
    engine: Option<SpeedTestEngine>,
    rx: Option<Receiver<TestEvent>>,
    info_rx: Option<Receiver<ConnectionInfo>>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            phase: Phase::Idle,
            result: TestResult::default(),
            live_mbps: 0.0,
            info: None,
            error: None,
            engine: None,
            rx: None,
            info_rx: None,
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self.phase, Phase::Ping | Phase::Download | Phase::Upload)
    }

    pub fn lookup_info(&mut self, ctx: &egui::Context) {
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let agent = build_agent();
            if let Some(info) = fetch_connection_info(&agent) {
                let _ = tx.send(info);
                ctx.request_repaint();
            }
        });
        self.info_rx = Some(rx);
    }

    pub fn start(&mut self, ctx: &egui::Context) {
        let info = self.info.take();
        let info_rx = self.info_rx.take();
        *self = Self::new();
        self.info = info;
        self.info_rx = info_rx;

        let (tx, rx) = mpsc::channel();
        self.phase = Phase::Ping;
        self.engine = Some(SpeedTestEngine::start(tx, ctx.clone()));
        self.rx = Some(rx);
        self.poll();
    }

    pub fn stop(&mut self) {
        self.engine = None;
        self.rx = None;
        self.phase = Phase::Idle;
        self.live_mbps = 0.0;
    }

    pub fn poll(&mut self) {
        if let Some(rx) = &self.info_rx {
            if let Ok(info) = rx.try_recv() {
                self.info = Some(info);
            }
        }

        let events: Vec<TestEvent> = match &self.rx {
            Some(rx) => rx.try_iter().collect(),
            None => return,
        };

        for ev in events {
            match ev {
                TestEvent::PhaseChanged(p) => {
                    self.phase = p;
                    self.live_mbps = 0.0;
                }
                TestEvent::Connection(info) => self.info = Some(info),
                TestEvent::PingDone { ping_ms, jitter_ms } => {
                    self.result.ping_ms = Some(ping_ms);
                    self.result.jitter_ms = Some(jitter_ms);
                }
                TestEvent::LiveSpeed(v) => self.live_mbps = v,
                TestEvent::DownloadDone(v) => {
                    self.result.download_mbps = Some(v);
                    self.live_mbps = v;
                }
                TestEvent::UploadDone(v) => {
                    self.result.upload_mbps = Some(v);
                    self.live_mbps = v;
                }
                TestEvent::Failed(msg) => {
                    self.error = Some(msg);
                    self.phase = Phase::Failed;
                    self.live_mbps = 0.0;
                    self.engine = None;
                    self.rx = None;
                    return;
                }
                TestEvent::Finished => {
                    self.phase = Phase::Done;
                    self.live_mbps = self.result.download_mbps.unwrap_or(0.0);
                    self.engine = None;
                    self.rx = None;
                    return;
                }
            }
        }
    }
}
