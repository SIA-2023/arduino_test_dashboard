use eframe::egui;
use egui_plot::{Plot, Line, PlotPoint, PlotPoints};
use serial2::SerialPort;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, Default, Debug)]
struct Msg {
	sin_value: f32,
	cos_value: f32,
}

pub struct Dashboard {
	messages: [Msg; 60],
	current_message_index: usize,
	port: Option<serial2::SerialPort>,
	port_name: Option<std::path::PathBuf>,
	show_raw_data_window: bool,
	receiver: Option<std::sync::mpsc::Receiver<Msg>>,
}

impl Dashboard {
	pub fn new() -> Self {
		Self {
			messages: [Msg::default(); 60],
			current_message_index: 0,
			port: None,
			port_name: None,
			show_raw_data_window: false,
			receiver: None,
		}
	}

	pub fn ui(&mut self, ui: &mut egui::Ui) {
		ui.ctx().request_repaint(); // reactive mode doesn't react to new serial msgs
		
		if let Some(receiver) = &self.receiver {
			while let Ok(msg) = receiver.try_recv() {
				self.current_message_index = (self.current_message_index + 1) % self.messages.len();
				self.messages[self.current_message_index % self.messages.len()] = msg;
			}
		}

		// background
		ui.painter().rect_filled(egui::Rect::from_min_max(egui::pos2(0.0, 0.0), ui.ctx().screen_rect().size().to_pos2()), 0.0, egui::Color32::BLACK);

		if pick_port_ui(ui, &mut self.port_name) {
			match &self.port_name {
				Some(port_name) => {
					self.port = Some(SerialPort::open(port_name, 9600).unwrap());
					let port_clone = self.port.as_ref().unwrap().try_clone().unwrap();
					let (mpsc_sender, mpsc_receiver) = std::sync::mpsc::channel();
					self.receiver = Some(mpsc_receiver);
					std::thread::spawn(move || listen_thread(port_clone, mpsc_sender));
				},
				None => {
					self.port = None;
					self.receiver = None;
				}
			}
		}

		if !self.show_raw_data_window && ui.button("Show raw data window").clicked() {
			self.show_raw_data_window = true;
		}

		if let Some(port) = self.port.as_ref() {
			ui.label("Controls:");

			if ui.button("+").clicked() {
				let _ = port.write(b"+");
			}

			if ui.button("-").clicked() {
				let _ = port.write(b"-");
			}
		}

		show_plot(ui, "sin_value", &self.messages, self.current_message_index, |msg| msg.sin_value as f64);
		show_plot(ui, "cos_value", &self.messages, self.current_message_index, |msg| msg.cos_value as f64);

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
	if let Ok(ports) = SerialPort::available_ports() {
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
	ui.label(format!("{}:", name));

	Plot::new(name)
		.y_axis_width(2)
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

fn listen_thread(port: SerialPort, sender: std::sync::mpsc::Sender<Msg>) {
	let mut bytes: Vec<u8> = Vec::new();
	let mut current_byte = [0_u8];
	loop {
		match port.read(&mut current_byte) {
			Ok(bytes_read) => {
				if bytes_read == 1 {
					if current_byte[0] == b'\n' {
						if let Ok(msg) = bincode::deserialize(&bytes) {
							if sender.send(msg).is_err() {
								return; // the receiver was dropped -> close this thread
							}
						}
						
						bytes.clear();
					}
					else {
						bytes.push(current_byte[0]);
					}
				}
			},
			Err(e) => {
				eprintln!("error reading: {e}");
			}
		}
	}
}

