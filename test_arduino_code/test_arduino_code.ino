struct Msg {
	int32_t left_motor = 0;
	int32_t right_motor = 0;
	float kp = 0.f;
	float ki = 0.f;
	float kd = 0.f;
	bool left_sensor = false;
	bool right_sensor = false;
} msg;

struct Command {
	char target = '\0';
	float value = 0.f;
} command;

float kp = 0.f;
float ki = 0.f;
float kd = 0.f;

void setup() {
	Serial.begin(9600);
}

void loop() {
	float t = (float)millis() / 1000.f;

	// receive command
	if (Serial.available() && Serial.readBytesUntil('\n', (uint8_t*)&command, sizeof(command)) == sizeof(command)) {
		switch (command.target) {
			case 'p': kp = command.value; break;
			case 'i': ki = command.value; break;
			case 'd': kd = command.value; break;
		}
	}

	// send random example data
	msg.left_motor = (sin(t * 20.f) / 2.f + 0.5f) * 255;
	msg.right_motor = -(cos(t * 20.f) / 2.f + 0.5f) * 255;
	msg.left_sensor = (int)t % 2 == 0;
	msg.right_sensor = !msg.left_sensor;
	msg.kp = kp;
	msg.ki = ki;
	msg.kd = kd;
	Serial.write((const uint8_t*)&msg, sizeof(msg));
	Serial.write('\n');
}