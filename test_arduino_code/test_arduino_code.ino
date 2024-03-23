void setup() {
  Serial.begin(9600);
}

struct Msg {
  int32_t left_motor = 0;
  int32_t right_motor = 0;
  bool left_sensor = false;
  bool right_sensor = false;
};

void loop() {
  float t = (float)millis() / 1000.f;

  static float time_scale = 1.f;
  if (Serial.available()) {
    switch (Serial.read()) {
      case '+': time_scale += 0.1f; break;
      case '-': time_scale -= 0.1f; break;
    }
  }
  
  Msg msg{};
  msg.left_motor = (sin(t * 20.f * time_scale) / 2.f + 0.5f) * 255;
  msg.right_motor = -(cos(t * 20.f * time_scale) / 2.f + 0.5f) * 255;
  msg.left_sensor = (int)t % 2 == 0;
  msg.right_sensor = !msg.left_sensor;
  Serial.write((const uint8_t*)&msg, sizeof(msg));
  Serial.write('\n');
}

