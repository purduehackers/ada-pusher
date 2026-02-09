# ada-pusher

Companion to [door-opener][door-opener-github] to push the Bechtel ADA button

[door-opener-github]: https://github.com/purduehackers/door-opener


## Build

You need to install the [Rust on ESP32][esp32-rs-start] prerequisites. These
commands may be useful:

```
cargo binstall espup
espup install
. ~/export-esp.sh
cargo binstall espflash
cargo binstall ldproxy
```

Then, build with `cargo build`.

Connect the ESP32 board to your computer and run:

```
espflash flash --monitor target/xtensa-esp32-espidf/debug/ada-pusher
```

[esp32-rs-start]: https://docs.espressif.com/projects/rust/book/getting-started/index.html
