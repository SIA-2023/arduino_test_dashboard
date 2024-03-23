use eframe::egui;
use egui_plot::{Plot, Line, PlotPoint, PlotPoints};
use crate::dashboard::Msg;

pub fn circle(ui: &mut egui::Ui, radius: f32, color: egui::Color32) {
	ui.painter().circle(ui.cursor().min + egui::vec2(radius, radius), radius, color, egui::Stroke::new(1.0, egui::Color32::RED));
	ui.allocate_rect(egui::Rect::from_min_size(ui.cursor().min, egui::vec2(radius * 2.0, radius * 2.0)), egui::Sense::hover());
}

pub fn show_plot(ui: &mut egui::Ui, name: &str, messages: &[Msg], current_message_index: usize, callback: impl Fn(&Msg) -> f64) {
	Plot::new(name)
		.y_axis_width(3)
		.height(150.0)
		.allow_drag(false)
		.allow_zoom(false)
		.allow_boxed_zoom(false)
		.allow_scroll(false)
		.show(ui, |ui| {
			let mut points: Vec<PlotPoint> = Vec::with_capacity(messages.len());
			for i in (1..messages.len()+1).rev() {
				let index = (i + current_message_index) % messages.len();
				let value = callback(&messages[index]);
				points.push(PlotPoint::new(i as f64 - 1.0, value));
			}

			ui.line(
				Line::new(PlotPoints::Owned(points))
					.name(name)
			);
		});
}

pub fn vertical_percentage_bar(ui: &mut egui::Ui, mut percentage: f32, size: egui::Vec2) {
	percentage = percentage.clamp(0.0, 1.0);
	
	let rect = egui::Rect::from_min_size(ui.cursor().min, size);
	let percentage_rect = egui::Rect::from_min_size(rect.left_bottom() - egui::vec2(0.0, size.y * percentage), egui::vec2(size.x, size.y * percentage));
	ui.allocate_rect(rect, egui::Sense::hover());

	ui.painter().rect(rect, 0.0, egui::Color32::from_gray(80), egui::Stroke::new(1.0, egui::Color32::WHITE));
	ui.painter().rect_filled(percentage_rect, 0.0, egui::Color32::RED);
}

pub fn horizontal_percentage_bar(ui: &mut egui::Ui, mut percentage: f32, size: egui::Vec2) {
	percentage = percentage.clamp(0.0, 1.0);
	
	let rect = egui::Rect::from_min_size(ui.cursor().min, size);
	let percentage_rect = egui::Rect::from_min_size(rect.min, egui::vec2(size.x * percentage, size.y));
	ui.allocate_rect(rect, egui::Sense::hover());

	ui.painter().rect(rect, 0.0, egui::Color32::from_gray(80), egui::Stroke::new(1.0, egui::Color32::WHITE));
	ui.painter().rect_filled(percentage_rect, 0.0, egui::Color32::RED);
}

// speed: -1.0..1.0
pub fn wheel(ui: &mut egui::Ui, mut speed: f32, size: egui::Vec2) {
	speed = speed.clamp(-1.0, 1.0);

	let rect = egui::Rect::from_min_size(ui.cursor().min, size);
	ui.allocate_rect(rect, egui::Sense::hover());
	ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(80));

	let y_offset = size.y / 4.0;
	let x = rect.center().x;
	for i in 0..3 {
		let y = rect.min.y + (i + 1) as f32 * y_offset;
		let arrow_point = egui::pos2(x, y - 0.5 * y_offset * speed);
		let arrow_back_y = y + 0.5 * y_offset * speed;
		ui.painter().line_segment([egui::pos2(rect.min.x, arrow_back_y), arrow_point], egui::Stroke::new(1.0, egui::Color32::WHITE));
		ui.painter().line_segment([egui::pos2(rect.min.x + size.x, arrow_back_y), arrow_point], egui::Stroke::new(1.0, egui::Color32::WHITE));
	}
}