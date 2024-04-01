use eframe::egui;

mod dashboard;
use dashboard::Dashboard;

mod serial;

mod widgets;

fn main() -> Result<(), eframe::Error> {
	let viewport = egui::ViewportBuilder::default().with_inner_size([1200.0, 1050.0]);
	let options = eframe::NativeOptions{ viewport, ..Default::default() };
	let mut dashboard = Dashboard::new();

	eframe::run_simple_native("Test Dashboard", options, move |ctx, _frame| {
		egui::CentralPanel::default().show(ctx, |ui| {
			dashboard.ui(ui);
		});
	})
}