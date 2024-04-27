use eframe::egui;
use crate::serial::{Serial, Msg};
use crate::widgets::{circle, horizontal_percentage_bar, show_plot, vertical_percentage_bar, wheel};

const MESSAGES: usize = 500;

pub struct Dashboard {
	messages: [Msg; MESSAGES],
	raw_messages: Vec<String>,
	current_message_index: usize,
	serial: Option<Serial>,
	port_name: Option<std::path::PathBuf>,
	kp: f32,
	ki: f32,
	kd: f32,
}

impl Dashboard {
	pub fn new() -> Self {
		let mut raw_messages = Vec::with_capacity(MESSAGES);
		for _ in 0..MESSAGES {
			raw_messages.push(String::new());
		}
		Self {
			messages: [Msg::default(); MESSAGES],
			raw_messages,
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
			serial.collect_messages(|msg, raw_msg| {
				self.current_message_index = (self.current_message_index + 1) % self.messages.len();
				if let Some(msg) = msg {
					self.messages[self.current_message_index % self.messages.len()] = msg;
				}
				self.raw_messages[self.current_message_index] = raw_msg;
			});
		}

		// background
		ui.painter().rect_filled(egui::Rect::from_min_max(egui::pos2(0.0, 0.0), ui.ctx().screen_rect().size().to_pos2()), 0.0, egui::Color32::BLACK);

		if Self::pick_port_ui(ui, &mut self.port_name) {
			self.serial = self.port_name.as_ref().map(|port_name| Serial::new(port_name).unwrap());
		}
		ui.add_space(15.0);
		
		let current_msg = &self.messages[self.current_message_index];

		// sensors
		ui.horizontal(|ui| {
			ui.vertical(|ui| {
				ui.heading("left_sensor:");
				circle(ui, 25.0, if current_msg.left_sensor { egui::Color32::WHITE } else { egui::Color32::BLACK });
			});
			ui.add_space(40.0);
			ui.vertical(|ui| {
				ui.heading("right_sensor:");
				circle(ui, 25.0, if current_msg.right_sensor { egui::Color32::WHITE } else { egui::Color32::BLACK });
			});
		});
		ui.add_space(15.0);

		// constants
		if let Some(serial) = self.serial.as_ref() {
			ui.horizontal(|ui| {
				ui.vertical(|ui| {
					ui.heading("Controls:");
					if ui.add(egui::Slider::new(&mut self.kp, 0.0..=1.0).text("kp")).changed() {
						let _ = serial.set_value('p', self.kp);
					}
					if ui.add(egui::Slider::new(&mut self.ki, 0.0..=1.0).text("ki")).changed() {
						let _ = serial.set_value('i', self.ki);
					}
					if ui.add(egui::Slider::new(&mut self.kd, 0.0..=1.0).text("kd")).changed() {
						let _ = serial.set_value('d', self.kd);
					}
				});
				ui.add_space(40.0);
				ui.vertical(|ui| {
					ui.heading("Arduino PID-Constants:");
					ui.label(format!("kp = {}", current_msg.kp));
					ui.label(format!("ki = {}", current_msg.ki));
					ui.label(format!("kd = {}", current_msg.kd));
				});
			});
			ui.add_space(15.0);	
		}

		// motors
		ui.horizontal(|ui| {
			ui.vertical(|ui| {
				ui.heading("left_motor:");
				ui.horizontal(|ui| {
					vertical_percentage_bar(ui, current_msg.left_motor.abs() as f32 / 255.0, egui::vec2(30.0, 100.0));
					horizontal_percentage_bar(ui, current_msg.left_motor.abs() as f32 / 255.0, egui::vec2(100.0, 30.0));
					wheel(ui, current_msg.left_motor as f32 / 255.0, egui::vec2(30.0, 100.0));
				});
			});
			ui.add_space(40.0);
			ui.vertical(|ui| {
				ui.heading("right_motor:");
				ui.horizontal(|ui| {
					vertical_percentage_bar(ui, current_msg.right_motor.abs() as f32 / 255.0, egui::vec2(30.0, 100.0));
					horizontal_percentage_bar(ui, current_msg.right_motor.abs() as f32 / 255.0, egui::vec2(100.0, 30.0));
					wheel(ui, current_msg.right_motor as f32 / 255.0, egui::vec2(30.0, 100.0));
				});
			});
		});
		ui.add_space(15.0);
		

		let (left_rect, right_rect) = egui::Rect::from_min_size(ui.cursor().min, ui.available_size()).split_left_right_at_fraction(0.5);
		let mut left = ui.child_ui(left_rect, egui::Layout::default());
		let y_start = ui.cursor().min.y;
		show_plot(&mut left, "left_motor", &self.messages, self.current_message_index, |msg| msg.left_motor as f64);
		let mut right = ui.child_ui(right_rect, egui::Layout::default());		
		show_plot(&mut right, "right_motor", &self.messages, self.current_message_index, |msg| msg.right_motor as f64);
		ui.add_space(left.cursor().min.y - y_start);
		ui.add_space(15.0);

		ui.heading("raw messages:");
		egui::ScrollArea::vertical()
			.stick_to_bottom(true)
			.show(ui, |ui| {
				for i in 1..self.raw_messages.len()+1 {
					let index = (i + self.current_message_index) % self.raw_messages.len();
					ui.label(&self.raw_messages[index]);	
				}
			});
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
}