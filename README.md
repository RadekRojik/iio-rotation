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

## Usage

### Interactive mode

```bash
iio-rotation -c ~/.config/iio-rotation/hyprland.toml
```

---

### Override debounce from CLI

```bash
iio-rotation -d 500
```

---

### Test mode (`-e`)

Used to manually test configuration and scripts without waiting for D-Bus events.

```bash
iio-rotation -c hyprland.toml -e normal
iio-rotation -c hyprland.toml -e leftup
iio-rotation -c hyprland.toml -e rightup
iio-rotation -c hyprland.toml -e bottomup
```

In this mode:

* D-Bus is not used
* debounce is ignored
* only the selected command is executed

