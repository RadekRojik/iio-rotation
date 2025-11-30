# iio-rotation

`iio-rotation` is a small Linux utility that **listens to accelerometer events via D-Bus (`iio-sensors` / `SensorProxy`)** and **executes user-defined scripts based on device orientation changes**.

It is primarily intended for **automatic screen and input device rotation** on laptops, tablets, and convertibles — working on both **Wayland and X11**.

The project was **inspired by `iio-hyprland`**. The core idea is similar, but the solution is more general and not tied to any specific compositor.

---

## How it works

The program communicates with the system service:

* **D-Bus service:** `net.hadess.SensorProxy`
* **property:** `AccelerometerOrientation`

Possible orientation values:

* `normal`
* `left-up`
* `right-up`
* `bottom-up`
* `undefined`

Internally, these values are **normalized** (lowercase, hyphens removed) to:

* `normal`
* `leftup`
* `rightup`
* `bottomup`
* `undefined`

Based on the resulting value, the corresponding command from the configuration file is executed.

---

## Characteristics

* **no polling**
* **blocking D-Bus calls**
* minimal CPU usage

The program **does not run in a busy loop**.

As a result:

* CPU usage stays near zero
* suitable for long-running background use
* no periodic sensor querying is required

---

## Configuration

Configuration is done using the **TOML** format.

The program includes a built-in **default configuration**, but a custom config file can be specified using the `-c` argument.

Basic configuration structure:

```toml
debounce = 300

[orientation]
normal    = "command"
leftup    = "command"
rightup   = "command"
bottomup  = "command"
undefined = "command"
```

### `debounce`

Time in milliseconds.

Used to suppress rapid oscillations of the sensor — the script is executed only after the delay has elapsed and only if the orientation actually changed.

---

## Example configurations

### Hyprland (`hyprland.toml`)

```toml
debounce = 300

[orientation]
normal = "hyprctl keyword --batch \"keyword monitor eDP-1,transform,0; keyword input:touchdevice:transform 0;\""
leftup = "hyprctl keyword --batch \"keyword monitor eDP-1,transform,1; keyword input:touchdevice:transform 1;\""
rightup = "hyprctl keyword --batch \"keyword monitor eDP-1,transform,3; keyword input:touchdevice:transform 3;\""
bottomup = "hyprctl keyword --batch \"keyword monitor eDP-1,transform,2; keyword input:touchdevice:transform 2;\""
undefined = "notify-send -t 1000 'iio-rotation' 'Some undefined event (orientation unknown)'"
```

---

### Niri (`niri.toml`)

```toml
debounce = 300

[orientation]
normal   = "niri msg output eDP-1 transform normal"
leftup   = "niri msg output eDP-1 transform 90"
rightup  = "niri msg output eDP-1 transform 270"
bottomup = "niri msg output eDP-1 transform 180"
undefined = "notify-send -t 1000 'iio-rotation' 'Some undefined event (orientation unknown)'"
```

---

### wlroots / wlr-randr (`wlroots.toml`)

```toml
debounce = 300

[orientation]
normal   = "wlr-randr --output eDP-1 --transform normal"
leftup   = "wlr-randr --output eDP-1 --transform 90"
rightup  = "wlr-randr --output eDP-1 --transform 270"
bottomup = "wlr-randr --output eDP-1 --transform 180"
undefined = "notify-send -t 1000 'iio-rotation' 'Some undefined event (orientation unknown)'"
```

---

### X11 (`x11.toml`)

```toml
debounce = 300

[orientation]
normal = "xrandr --output eDP-1 --rotate normal; xinput set-prop \"Wacom Pen and multitouch sensor Finger touch\" --type=float \"Coordinate Transformation Matrix\" 1 0 0 0 1 0 0 0 1"
rightup = "xrandr --output eDP-1 --rotate right; xinput set-prop \"Wacom Pen and multitouch sensor Finger touch\" --type=float \"Coordinate Transformation Matrix\" 0 1 0 -1 0 1 0 0 1"
leftup = "xrandr --output eDP-1 --rotate left; xinput set-prop \"Wacom Pen and multitouch sensor Finger touch\" --type=float \"Coordinate Transformation Matrix\" 0 -1 1 1 0 0 0 0 1"
bottomup = "xrandr --output eDP-1 --rotate inverted; xinput set-prop \"Wacom Pen and multitouch sensor Finger touch\" --type=float \"Coordinate Transformation Matrix\" -1 0 1 0 -1 1 0 0 1"
undefined = "notify-send -t 1000 'iio-rotation' 'Some undefined event (orientation unknown)'"
```

---

# iio-rotation – build & configuration guide

This describes how to compile `iio-rotation` from source and where to place configuration files.

---

## Requirements

* Linux system with D-Bus
* `iio-sensor-proxy` running (usually provided by the distribution)
* Rust toolchain (stable)

On most distributions, `iio-sensor-proxy` is already installed on laptops or tablets with an accelerometer.

---

## Installing Rust

If Rust is not installed, the recommended way is via rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then restart the shell or run:

```bash
source ~/.cargo/env
```

---

## Building the program

Clone the repository:

```bash
git clone https://codeberg.org/ramael/iio-rotation.git
cd iio-rotation
```

Compile in release mode:

```bash
cargo build --release
```

The resulting binary will be located at:

```
target/release/iio-rotation
```

Optionally install it system-wide:

```bash
sudo install -Dm755 target/release/iio-rotation /usr/local/bin/iio-rotation
```

---

## Configuration files

### Default behavior

If no configuration file is specified using the `-c` option, the program first tries to load the configuration from:

```
~/.config/iio-rotation/config.toml
```

If this file does not exist, the built-in default configuration is used instead.

---

### User configuration directory

Recommended location for user configuration files:

```
~/.config/iio-rotation/
```

Example:

```
~/.config/iio-rotation/hyprland.toml
~/.config/iio-rotation/niri.toml
~/.config/iio-rotation/wlroots.toml
~/.config/iio-rotation/x11.toml
```

Configuration files are normal TOML files and can be freely edited by the user.

---

### Selecting a configuration file

Use the `-c` option to select which configuration file to load:

```bash
iio-rotation -c ~/.config/iio-rotation/hyprland.toml
```

The path is relative to the user’s home directory if not absolute.

---

## Running the program

### Interactive mode (default)

In this mode, the program listens for accelerometer events via D-Bus and reacts to orientation changes:

```bash
iio-rotation -c ~/.config/iio-rotation/hyprland.toml
```

The program blocks while waiting for events and does not consume CPU time unnecessarily.

---

### Overriding debounce time

The debounce time defined in the configuration file can be overridden from the command line:

```bash
iio-rotation -d 500
```

The value is in milliseconds.

---

### Test mode

Test mode allows manual testing of configuration files and scripts without using D-Bus.

Example:

```bash
iio-rotation -c hyprland.toml -e normal
iio-rotation -c hyprland.toml -e leftup
iio-rotation -c hyprland.toml -e rightup
iio-rotation -c hyprland.toml -e bottomup
```

In test mode:

* D-Bus is not used
* debounce is ignored
* only the selected command is executed

This is useful for validating scripts and configuration logic.

---

## Notes

* The program automatically claims and releases the accelerometer via `SensorProxy`.
* All commands from the configuration file are executed via `sh -c`.
* Shell features such as chaining commands or calling external scripts are supported.

