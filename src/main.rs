#![windows_subsystem = "windows"]
mod speedtest;
mod ui;

use ui::window::SpeedTestWindow;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([420.0, 760.0])
            .with_min_inner_size([380.0, 680.0])
            .with_resizable(true)
            .with_title("Speed Test"),
        ..Default::default()
    };

    eframe::run_native(
        "speedtest",
        options,
        Box::new(|cc| Box::new(SpeedTestWindow::new(cc))),
    )
}
