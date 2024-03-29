float kp = 0.f;
float ki = 0.f;
float kd = 0.f;

void setup() {
	Serial.begin(115200);
}

void loop() {
	// receive command
	if (Serial.available()) {
		// format: "{target}{value}\n";
		String command = Serial.readStringUntil('\n');
		char target = command.charAt(0);
		float value = command.substring(1).toFloat();
		switch (target) {
			case 'p': kp = value; break;
			case 'i': ki = value; break;
			case 'd': kd = value; break;
		}
	}

	// send random example data
	float t = (float)millis() / 1000.f;
	int left_motor = (sin(t * 20.f) / 2.f + 0.5f) * 255;
	int right_motor = -(cos(t * 20.f) / 2.f + 0.5f) * 255;
	bool left_sensor = (int)t % 2 == 0;
	bool right_sensor = !left_sensor;
	
	// format: "{left_motor},{right_motor},{left_sensor},{right_sensor},{kp},{ki},{kd}\n"
	Serial.print(left_motor);
	Serial.print(',');
	Serial.print(right_motor);
	Serial.print(',');
	Serial.print(left_sensor);
	Serial.print(',');
	Serial.print(right_sensor);
	Serial.print(',');
	Serial.print(kp);
	Serial.print(',');
	Serial.print(ki);
	Serial.print(',');
	Serial.print(kd);
	Serial.print('\n');
}