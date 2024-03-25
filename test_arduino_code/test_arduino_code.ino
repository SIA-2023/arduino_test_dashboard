void setup() {
  Serial.begin(9600);
}

struct Msg {
  int32_t left_motor = 0;
  int32_t right_motor = 0;
  float kp = 0.f;
  float ki = 0.f;
  float kd = 0.f;
  bool left_sensor = false;
  bool right_sensor = false;
};

void loop() {
  float t = (float)millis() / 1000.f;

  static float time_scale = 1.f;
  static float kp = 0.f;
  static float ki = 0.f;
  static float kd = 0.f;
  if (Serial.available()) {
    // command: "p1.0\n" -> change kp to 1.0
    // in arduino ide: type "p1.0" and hit enter
    String command = Serial.readStringUntil('\n');
    float value = command.substring(1).toFloat();
    switch (command.charAt(0)) {
    case 'p': kp = value; break;
    case 'i': ki = value; break;
    case 'd': kd = value; break;
    }
  }
  
  Msg msg{};
  msg.left_motor = (sin(t * 20.f * time_scale) / 2.f + 0.5f) * 255;
  msg.right_motor = -(cos(t * 20.f * time_scale) / 2.f + 0.5f) * 255;
  msg.left_sensor = (int)t % 2 == 0;
  msg.right_sensor = !msg.left_sensor;
  msg.kp = kp;
  msg.ki = ki;
  msg.kd = kd;
  Serial.write((const uint8_t*)&msg, sizeof(msg));
  Serial.write('\n');
}