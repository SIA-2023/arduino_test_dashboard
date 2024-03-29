use std::path::PathBuf;
use serial2::SerialPort;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, Default, Debug)]
#[repr(C)]
pub struct Msg {
	pub left_motor: i32,
	pub right_motor: i32,
	pub kp: f32,
	pub ki: f32,
	pub kd: f32,
	pub left_sensor: bool,
	pub right_sensor: bool,
}

pub struct Serial {
	port: SerialPort,
	port_name: PathBuf,
	receiver: std::sync::mpsc::Receiver<Msg>,
}

impl Serial {
	pub fn new(port_name: &PathBuf) -> std::io::Result<Self> {
		let port = SerialPort::open(port_name, 9600)?;
		let port_clone = port.try_clone()?;
		let (sender, receiver) = std::sync::mpsc::channel();
		std::thread::spawn(move || Self::listen_thread(port_clone, sender));
		
		Ok(Self {
			port,
			port_name: port_name.clone(),
			receiver,
		})
	}

	pub fn collect_messages(&mut self, mut callback: impl FnMut(Msg)) {
		while let Ok(msg) = self.receiver.try_recv() {
			callback(msg);
		}
	}

	pub fn set_value(&self, target: char, value: f32) -> std::io::Result<usize> {
		self.port.write(format!("{target}{value}\n").as_bytes())
	}

	pub fn is_connected(&self) -> bool {
		if let Ok(ports) = SerialPort::available_ports() {
			ports.contains(&self.port_name)
		}
		else {
			false
		}
	}

	pub fn available_ports() -> std::io::Result<Vec<PathBuf>> {
		SerialPort::available_ports()
	}

	fn parse_msg(input: &str) -> Option<Msg> {
		let parts: Vec<&str> = input.split(',').collect();
		if parts.len() != 7 {
			return None;
		}
	
		let left_motor = parts[0].parse::<i32>().ok()?;
		let right_motor = parts[1].parse::<i32>().ok()?;
		let left_sensor = parts[2].parse::<i32>().ok()? == 1;
		let right_sensor = parts[3].parse::<i32>().ok()? == 1;
		let kp = parts[4].parse::<f32>().ok()?;
		let ki = parts[5].parse::<f32>().ok()?;
		let kd = parts[6].parse::<f32>().ok()?;
	
		Some(Msg {
			left_motor,
			right_motor,
			left_sensor,
			right_sensor,
			kp,
			ki,
			kd,
		})
	}

	fn listen_thread(port: SerialPort, sender: std::sync::mpsc::Sender<Msg>) {
		let mut msg = String::new();
		let mut current_byte = [0_u8];
		loop {
			match port.read(&mut current_byte) {
				Ok(bytes_read) => {
					if bytes_read == 1 {
						if current_byte[0] == b'\n' {
							if let Some(msg) = Self::parse_msg(&msg) {
								if sender.send(msg).is_err() {
									return; // the receiver was dropped -> close this thread
								}
							}
							
							msg.clear();
						}
						else {
							msg.push(current_byte[0] as char);
						}
					}
				},
				Err(e) => {
					eprintln!("error reading: {e}");
				}
			}
		}
	}	
}