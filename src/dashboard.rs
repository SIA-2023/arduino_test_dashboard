use eframe::egui;
use crate::serial::{Serial, Msg};
use crate::widgets::{circle, horizontal_percentage_bar, show_plot, vertical_percentage_bar, wheel};

pub struct Dashboard {
	messages: [Msg; 60],
	current_message_index: usize,
	serial: Option<Serial>,
	port_name: Option<std::path::PathBuf>,
	kp: f32,
	ki: f32,
	kd: f32,
}

impl Dashboard {
	pub fn new() -> Self {
		Self {
			messages: [Msg::default(); 60],
			current_message_index: 0,
			serial: None,
			port_name: None,
			kp: 0.0,
			ki: 0.0,
			kd: 0.0,
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

		if let Some(serial) = self.serial.as_ref() {
			ui.label("Controls:");

			if ui.add(egui::Slider::new(&mut self.kp, 0.0..=1.0).text("kp")).changed() {
				let _ = serial.set_value('p', self.kp);
			}
			if ui.add(egui::Slider::new(&mut self.ki, 0.0..=1.0).text("ki")).changed() {
				let _ = serial.set_value('i', self.ki);
			}
			if ui.add(egui::Slider::new(&mut self.kd, 0.0..=1.0).text("kd")).changed() {
				let _ = serial.set_value('d', self.kd);
			}
		}

		let current_msg = &self.messages[self.current_message_index];

		ui.label(format!("kp = {}", current_msg.kp));
		ui.label(format!("ki = {}", current_msg.ki));
		ui.label(format!("kd = {}", current_msg.kd));
		
		ui.label("left_motor:");
		ui.horizontal(|ui| {
			vertical_percentage_bar(ui, current_msg.left_motor.abs() as f32 / 255.0, egui::vec2(30.0, 100.0));
			horizontal_percentage_bar(ui, current_msg.left_motor.abs() as f32 / 255.0, egui::vec2(100.0, 30.0));
			wheel(ui, current_msg.left_motor as f32 / 255.0, egui::vec2(30.0, 100.0));
		});
		show_plot(ui, "left_motor", &self.messages, self.current_message_index, |msg| msg.left_motor as f64);
		
		ui.label("right_motor:");
		ui.horizontal(|ui| {
			vertical_percentage_bar(ui, current_msg.right_motor.abs() as f32 / 255.0, egui::vec2(30.0, 100.0));
			horizontal_percentage_bar(ui, current_msg.right_motor.abs() as f32 / 255.0, egui::vec2(100.0, 30.0));
			wheel(ui, current_msg.right_motor as f32 / 255.0, egui::vec2(30.0, 100.0));
		});
		show_plot(ui, "right_motor", &self.messages, self.current_message_index, |msg| msg.right_motor as f64);
		
		ui.label("left_sensor:");
		circle(ui, 25.0, if current_msg.left_sensor { egui::Color32::WHITE } else { egui::Color32::BLACK });

		ui.label("right_sensor:");
		circle(ui, 25.0, if current_msg.right_sensor { egui::Color32::WHITE } else { egui::Color32::BLACK });
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