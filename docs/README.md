
# Documentation

Welcome to the `ada-pusher` documentation.

## BOM

| Name | URL | Notes |
|------|-----|-------|
| ESP32 | The cheapest one you can find | `ESP32-WROOM-32D`, see notes below |
| L298N | [Amazon](https://www.amazon.com/dp/B0C5JCF5RS) | Motor driver/controller for linear actuator |
| LM2596 | [Amazon](https://www.amazon.com/dp/B0DBVYP91F) | Buck converter (15V -> 12V), suitable for motors and high power applications |
| Mini360 | [Amazon](https://www.amazon.com/dp/B08HQDSQZP) | Buck converter (15V -> 5V), suitable for low power applications such as the ESP32 board |
| USB-C PD Trigger Board | [Amazon](https://www.amazon.com/dp/B0CFTXRHLV) | Any trigger board should work, just make sure it supports 15V profile at minimum! |



### Notes

- For the ESP32, if replacing, try to get one with the RISC-V architecture
  - Implementation `v1` uses `ESP32-WROOM-32D`, which is based on Xtensa, and while it works, there are some annoyances where you have to install a modified toolchain and LLVM compiler from Espressif.
  - Only get Xtensa-based boards if they are significantly cheaper than the RISC-V-based ones.

## Circuit

![Circuit diagram](./wiring-diagram/image.png)

- Make sure to remove both jumpers from the `ENA` and `ENB` pins of the `L298N` driver.
- Adjust the potentiometer on the Mini360 until the output voltage is 5V when input is 15V.
- Adjust the potentiometer on the LM2596 until the output voltage is 12V when input is 15V.
- For the USB-C trigger board:
  - Most USB-C chargers/power adapters do not support the 12V profile, since the USB specification has deemed it optional. Therefore, we get 15V and drop it down to the required voltages, which is probably preferred as the power is more stable overall.
  - The particular board from Amazon comes with a tiny resistor between the 12V profile resistor and ground. You **must** remove this resistor, or else you may get a 9V output.
  - Solder the 15V profile resistor and the ground pad next to it together.
  - Verify 15V output when connected to a capable 15V USB-C PD source with a multimeter.
  - **Warning**: connecting a low-power USB-C PD source or a non-PD USB-C power source will result in the voltage dropping down to the highest voltage that the PD source can supply, which will usually be 5V or 9V. `ada-pusher` will not work with those voltages.
