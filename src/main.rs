// Program iio-rotation read from dbus values iio-sensors of Accelerometer
// and launch [scripts|programs] from user config
//
// Inspired by iio-hyprland, thanks a lot.
//
// Author Radek Rojík aka Ramael
//_______________________________
// d-bus adress
//
// /net/hadess/SensorProxy
// net.hadess.SensorProxy
// methods:
//     ClaimAccelerometer()
//     ReleaseAccelerometer()
// properties:
//     AccelerometerOrientation
//     results:
//         undefined
//         normal
//         bottom-up
//         left-up
//         right-up
//
//

use std::collections::HashMap;
use std::time::Duration;
use std::thread;
use std::path::PathBuf;
use std::process::Command;
use zbus::Result;
use zbus::blocking::Connection;
use zbus::proxy;
use zbus::zvariant::OwnedValue;
use serde::Deserialize;
use toml::Value;
use clap::Parser;


// arg parsing
#[derive(Parser)]
#[command(name = "iio-rotation")]
#[command(version, about, long_about = None)]
#[command(next_line_help = true)]

struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    /// Sets a custom debounce time in ms
    #[arg(short, long, value_name = "TIME")]
    debounce: Option<u64>,
    /// Invoke event
    #[arg(short, long, value_name = "undefined|normal|bottomup|leftup|rightup")]
    event: Option<String>,
}

// Set on SensorProxy properties dbus
#[proxy(
    interface = "net.hadess.SensorProxy",
    default_service = "net.hadess.SensorProxy",
    default_path = "/net/hadess/SensorProxy"
)]
trait SensorProxy {
    fn claim_accelerometer(&self) -> zbus::Result<()>;
    fn release_accelerometer(&self) -> zbus::Result<()>;
    #[zbus(property)]
    fn accelerometer_orientation(&self) -> zbus::Result<String>;
}


struct MySensorProxy<'a> {
    proxy: SensorProxyProxyBlocking<'a>,
}

impl<'a> MySensorProxy<'a> {
    fn new(connection: &'a Connection) -> zbus::Result<Self> {
        let proxy = SensorProxyProxyBlocking::new(connection)?;
        proxy.claim_accelerometer()?;
        Ok(Self { proxy })
    }

    fn proxy(&self) -> &SensorProxyProxyBlocking<'a> {
        &self.proxy
    }
}

// Implementatio Drop for release resources
impl<'a> Drop for MySensorProxy<'a> {
    fn drop(&mut self) {
        if let Err(e) = self.proxy.release_accelerometer() {
            eprintln!("Unable release_accelerometer: {e}");
        }
    }
}


// Defaul config
const DEFAULT_CONFIG_TOML: &str = r#"
# default debounce
debounce = 300

[orientation]
normal = "msg='normal orientation'; printf '%s\n' \"$msg\" | systemd-cat -t iio-rotation 2>/dev/null || logger -t iio-rotation -p user.notice -- \"$msg\""
leftup = "msg='leftup orientation'; printf '%s\n' \"$msg\" | systemd-cat -t iio-rotation 2>/dev/null || logger -t iio-rotation -p user.notice -- \"$msg\""
rightup = "msg='rightup orientation'; printf '%s\n' \"$msg\" | systemd-cat -t iio-rotation 2>/dev/null || logger -t iio-rotation -p user.notice -- \"$msg\""
bottomup = "msg='bottomup orientation'; printf '%s\n' \"$msg\" | systemd-cat -t iio-rotation 2>/dev/null || logger -t iio-rotation -p user.notice -- \"$msg\""
undefined = "msg='undefined orientation'; printf '%s\n' \"$msg\" | systemd-cat -t iio-rotation 2>/dev/null || logger -t iio-rotation -p user.notice -- \"$msg\""
"#;


#[derive(Debug, Deserialize)]
struct Config {
    debounce: u64,
    orientation: Orientation,
}

#[derive(Deserialize, Debug)]
struct Orientation {
    normal:    String,
    undefined: String,
    leftup:    String,
    rightup:   String,
    bottomup:  String,
}


fn merge_with_fallback(base: &mut Value, override_v: &Value, path: &str) {
    use toml::Value::*;

    match (base, override_v) {
        (Table(base_table), Table(override_table)) => {
            for (key, override_val) in override_table {
                let new_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };

                match base_table.get_mut(key) {
                    Some(base_val) => {
                        if matches!(base_val, Table(_)) && matches!(override_val, Table(_)) {
                            merge_with_fallback(base_val, override_val, &new_path);
                        } else if std::mem::discriminant(base_val) == std::mem::discriminant(override_val) {
                            *base_val = override_val.clone();
                        } else {
                            eprintln!(
                                "Config error {new_path}: expect type {}, but found {}. Use default.",
                                type_name(base_val),
                                type_name(override_val)
                            );
                        }
                    }
                    None => {
                        base_table.insert(key.clone(), override_val.clone());
                    }
                }
            }
        }
        _ => {
            // Nope
        }
    }
}

// helper for better naming
fn type_name(v: &Value) -> &'static str {
    match v {
        Value::String(_) => "String",
        Value::Integer(_) => "Integer",
        Value::Float(_) => "Float number",
        Value::Boolean(_) => "Boolean",
        Value::Array(_) => "Array",
        Value::Table(_) => "Table",
        Value::Datetime(_) => "Date/Time",
    }
}

fn load_config(configpath: PathBuf) -> Config {
    let mut config_path = dirs::home_dir().expect("Unable find home directory");
    config_path.push(configpath);

    let mut base: toml::Value = toml::from_str(DEFAULT_CONFIG_TOML)
        .expect("Error default TOML");

    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            match toml::from_str::<toml::Value>(&content) {
                Ok(user_val) => {
                    merge_with_fallback(&mut base, &user_val, "");
                }
                Err(e) => {
                    eprintln!("Parse error {:?}, ignore it: {e}", config_path);
                }
            }
        } else {
            eprintln!("Config exist, but unable read.");
        }
    }

    let cfg: Config = base
        .try_into()
        .expect("Default TOML not according with Config");

    cfg
}

/// Delete non alphanum chars
fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

fn doit(mezi: &String, config: &Config) {
        let command = match mezi.as_str() {
            "normal"    => config.orientation.normal.clone(),
            "bottomup"  => config.orientation.bottomup.clone(),
            "leftup"    => config.orientation.leftup.clone(),
            "rightup"   => config.orientation.rightup.clone(),
            _           => config.orientation.undefined.clone(),
        };
        let _output = Command::new("sh")
                        .arg("-c")
                        .arg(command)
                        .spawn()
                        .expect("failed to execute process");
    }


// *****************   main   *****************

fn main() -> Result<()> {
    let cli =Cli::parse();
    
    let configpath = cli.config.unwrap_or(".config/iio-rotation/config.toml".into());
    let mut config: Config = load_config(configpath);

    if cli.event.is_some(){
        let event = cli.event.unwrap();
        let _command  = doit(&event, &config);
        Ok(())
    } else {
        
        if cli.debounce.is_some(){
            config.debounce = cli.debounce.unwrap_or(300);
        }

    let connection = Connection::system()?;
    let accel = MySensorProxy::new(&connection)?;
    let proxy = accel.proxy();


    proxy.claim_accelerometer()?;

    let mut last = proxy.accelerometer_orientation()?;

    let props = zbus::blocking::Proxy::new(
        &connection,
        "net.hadess.SensorProxy",
        "/net/hadess/SensorProxy",
        "org.freedesktop.DBus.Properties",
    )?;

    let mut stream = props.receive_signal("PropertiesChanged")?;

    loop {
        let msg = stream.next().unwrap(); // blokuje, dokud něco nepřijde

        let (_iface, changed, _invalidated):
            (String, HashMap<String, OwnedValue>, Vec<String>) = msg.body().deserialize()?;


        if let Some(_val) = changed.get("AccelerometerOrientation") {
                thread::sleep(Duration::from_millis(config.debounce));

                let current = proxy.accelerometer_orientation()?;
                if current != last {
                    let mezi = normalize(&current.as_str());
                    let _command  = doit(&mezi, &config);
                    last = current;
                }
        }
    }
}
}
