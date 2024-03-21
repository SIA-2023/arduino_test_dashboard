use crate::dashboard::Msg;
use std::path::PathBuf;
use serial2::SerialPort;

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

	pub fn write(&self, bytes: &[u8]) -> std::io::Result<usize> {
		self.port.write(bytes)
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
}