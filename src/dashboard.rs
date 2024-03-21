use eframe::egui;
use egui_plot::{Plot, Line, PlotPoint, PlotPoints};
use serde::{Serialize, Deserialize};
use crate::serial::Serial;

#[derive(Serialize, Deserialize, Clone, Copy, Default, Debug)]
pub struct Msg {
	left_motor: i32,
	right_motor: i32,
}

pub struct Dashboard {
	messages: [Msg; 60],
	current_message_index: usize,
	serial: Option<Serial>,
	port_name: Option<std::path::PathBuf>,
	show_raw_data_window: bool,
}

impl Dashboard {
	pub fn new() -> Self {
		Self {
			messages: [Msg::default(); 60],
			current_message_index: 0,
			serial: None,
			port_name: None,
			show_raw_data_window: false,
		}
	}

	pub fn ui(&mut self, ui: &mut egui::Ui) {
		ui.ctx().request_repaint(); // reactive mode doesn't react to new serial msgs
		
		let serial_connected = if let Some(serial) = &self.serial {
			serial.is_connected()
		}
		else {
			false
		};
		if !serial_connected && self.serial.is_some() {
			self.serial = None;
			self.port_name = None;
		}

		if let Some(serial) = &mut self.serial {
			serial.collect_messages(|msg| {
				self.current_message_index = (self.current_message_index + 1) % self.messages.len();
				self.messages[self.current_message_index % self.messages.len()] = msg;
			});
		}

		// background
		ui.painter().rect_filled(egui::Rect::from_min_max(egui::pos2(0.0, 0.0), ui.ctx().screen_rect().size().to_pos2()), 0.0, egui::Color32::BLACK);

		if pick_port_ui(ui, &mut self.port_name) {
			self.serial = self.port_name.as_ref().map(|port_name| Serial::new(port_name).unwrap());
		}

		if !self.show_raw_data_window && ui.button("Show raw data window").clicked() {
			self.show_raw_data_window = true;
		}

		if let Some(serial) = self.serial.as_ref() {
			ui.label("Controls:");

			if ui.button("+").clicked() {
				let _ = serial.write(b"+");
			}

			if ui.button("-").clicked() {
				let _ = serial.write(b"-");
			}
		}

		ui.label("left_motor:");
		ui.horizontal(|ui| {
			vertical_percentage_bar(ui, self.messages[self.current_message_index].left_motor as f32 / 255.0, egui::vec2(30.0, 100.0));
			horizontal_percentage_bar(ui, self.messages[self.current_message_index].left_motor as f32 / 255.0, egui::vec2(100.0, 30.0));
		});
		show_plot(ui, "left_motor", &self.messages, self.current_message_index, |msg| msg.left_motor as f64);
		
		ui.label("right_motor:");
		ui.horizontal(|ui| {
			vertical_percentage_bar(ui, self.messages[self.current_message_index].right_motor as f32 / 255.0, egui::vec2(30.0, 100.0));
			horizontal_percentage_bar(ui, self.messages[self.current_message_index].right_motor as f32 / 255.0, egui::vec2(100.0, 30.0));
		});
		show_plot(ui, "right_motor", &self.messages, self.current_message_index, |msg| msg.right_motor as f64);
		
		egui::Window::new("Raw data window")
			.open(&mut self.show_raw_data_window)
			.show(ui.ctx(), |ui| {
				Self::raw_data_ui(ui, &self.messages[self.current_message_index]);
			});
	}

	fn raw_data_ui(ui: &mut egui::Ui, msg: &Msg) {
		ui.label(format!("{:#?}", msg));
	}
}

// returns wether port changed
fn pick_port_ui(ui: &mut egui::Ui, port_name: &mut Option<std::path::PathBuf>) -> bool {
	if let Ok(ports) = Serial::available_ports() {
		let mut changed = false;
		egui::ComboBox::from_label("Pick port")
			.selected_text(port_name.clone().unwrap_or(std::path::PathBuf::from("Pick port")).to_str().unwrap())
			.show_ui(ui, |ui| {
				if ui.selectable_value(port_name, None, "None").changed() {
					changed = true;
				}

				for name in ports {
					if ui.selectable_value(port_name, Some(name.clone()), name.to_str().unwrap()).changed() {
						changed = true;
					}
				}
			});
		changed
	}
	else {
		ui.colored_label(egui::Color32::RED, "Failed to get ports");
		false
	}
}

fn show_plot(ui: &mut egui::Ui, name: &str, messages: &[Msg], current_message_index: usize, callback: impl Fn(&Msg) -> f64) {
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

fn vertical_percentage_bar(ui: &mut egui::Ui, mut percentage: f32, size: egui::Vec2) {
	percentage = percentage.clamp(0.0, 1.0);
	
	let rect = egui::Rect::from_min_size(ui.cursor().min, size);
	let percentage_rect = egui::Rect::from_min_size(rect.left_bottom() - egui::vec2(0.0, size.y * percentage), egui::vec2(size.x, size.y * percentage));
	ui.allocate_rect(rect, egui::Sense::hover());

	ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(80));
	ui.painter().rect_filled(percentage_rect, 0.0, egui::Color32::RED);
}

fn horizontal_percentage_bar(ui: &mut egui::Ui, mut percentage: f32, size: egui::Vec2) {
	percentage = percentage.clamp(0.0, 1.0);
	
	let rect = egui::Rect::from_min_size(ui.cursor().min, size);
	let percentage_rect = egui::Rect::from_min_size(rect.min, egui::vec2(size.x * percentage, size.y));
	ui.allocate_rect(rect, egui::Sense::hover());

	ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(80));
	ui.painter().rect_filled(percentage_rect, 0.0, egui::Color32::RED);
}

