use eframe::egui;

mod dashboard;
use dashboard::Dashboard;

mod serial;

fn main() -> Result<(), eframe::Error> {
	let options = eframe::NativeOptions::default();
	let mut dashboard = Dashboard::new();

	eframe::run_simple_native("Dashboard", options, move |ctx, _frame| {
		egui::CentralPanel::default().show(ctx, |ui| {
			dashboard.ui(ui);
		});
	})
}