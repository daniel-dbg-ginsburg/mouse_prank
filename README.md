 # April Fools' BLE mouse

 A BLE mouse firmware that randomly jerks the cursor once in a while. Runs on ESP32-C3 (and probably also will on other ESP32s).

 Get yourself an ESP32-C3 board off Aliexpress, flash the firmware, pair it with your target's PC (some social engineering required), and leave it plugged somewhere around their desk. Watch the target troubleshoot.

 ## Installation 

Prerequisites: see https://github.com/esp-rs/esp-idf-template/blob/master/README.md#prerequisites (skip the espup part, C3 chip is RISC-V)

Connect the board to your computer and
 ```sh
 cargo run
 ```
