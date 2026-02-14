# ada-pusher

Companion to [door-opener][door-opener-github] to push the Bechtel ADA button

[door-opener-github]: https://github.com/purduehackers/door-opener

## Links

- [Documentation](./docs/README.md)

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

Create a `.env` file with the following contents:

```
PAIRING_PIN=123456
```

This is the pairing PIN used by `ada-pusher`. Replace with a suitable 6-digit PIN.

Then, build with `cargo build`.

Connect the ESP32 board to your computer and run:

```
espflash flash --monitor target/xtensa-esp32-espidf/debug/ada-pusher
```

[esp32-rs-start]: https://docs.espressif.com/projects/rust/book/getting-started/index.html
