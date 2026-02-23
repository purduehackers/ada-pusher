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

## Troubleshooting

### Max path on Windows

On Windows, you may encounter the following error:

```
error: failed to run custom build command for `esp-idf-sys v0.36.1`

Caused by:
  process didn't exit successfully: `C:\Users\user\Projects\ada-pusher\target\debug\build\esp-idf-sys-585097b028c03f00\build-script-build` (exit code: 1)
  --- stderr
  Error: Too long output directory: `\\?\C:\Users\user\Projects\ada-pusher\target\xtensa-esp32-espidf\debug\build\esp-idf-sys-7336b0b778f8f8a0\out`. Shorten your project path down to no more than 10 characters (or use WSL2 and its native Linux filesystem). Note that tricks like Windows `subst` do NOT work!
warning: build failed, waiting for other jobs to finish...
```

This is because you need to explicitly opt in to long file paths on Windows.
Recent versions of Windows enable long paths by default, but you need to let
ESP-IDF know of this. Set the environment variable:

```
$env:ESP_IDF_PATH_ISSUES = "warn"
```

