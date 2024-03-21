void setup() {
  Serial.begin(9600);
}

struct Msg {
  float sin_value = 0.f;
  float cos_value = 0.f;
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
  msg.sin_value = sin(t * 20.f * time_scale);
  msg.cos_value = cos(t * 20.f * time_scale);
  Serial.write((const uint8_t*)&msg, sizeof(msg));
  Serial.write('\n');
}

