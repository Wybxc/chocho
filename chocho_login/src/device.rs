//! 解析和生成 `device.json`。
//!
//! # Examples
//!
//! ```
//! use chocho_login::device::{from_json, random_from_uin};
//!
//! # fn main() -> anyhow::Result<()> {
//! let json = r#"{
//!     "deviceInfoVersion": 2,
//!     "data": {
//!         "display": "ROG Phone 3",
//!         "product": "ASUS_I003DD",
//!         "device": "ASUS_I003DD",
//!         "board": "sm8250",
//!         "model": "ASUS_I003DD",
//!         "fingerPrint": "google/redfin/redfin:11/RQ3A.210805.001.A1/7474174:user/release-keys",
//!         "bootId": "a7d7-0a00-0a00-0a00-000000000000",
//!         "procVersion": "Linux version 4.14.117-perf+ (hudsoncm@ilclbld72) (gcc version 4.9.x 20150123 (prerelease) (GCC)) #1 SMP PREEMPT Wed Jul 28 22:02:56 CST 2021",
//!         "brand": "asus",
//!         "bootloader": "PRB_A0_2005_10",
//!         "baseBand": "M3.0.50.1.31",
//!         "version": "11",
//!         "simInfo": "0",
//!         "osType": "android",
//!         "macAddress": "02:00:00:00:00:00",
//!         "ipAddress": "0a000103",
//!         "wifiBSSID": "02:00:00:00:00:00",
//!         "wifiSSID": "SSID",
//!         "imei": "280496984206895",
//!         "imsiMd5": "3e20e2c552e4a01c43cd7c802310b778",
//!         "androidId": "e55745001ab98456",
//!         "apn": "wifi",
//!         "vendorName": "asus",
//!         "vendorOsName": "WW",
//!         "version": {
//!             "codename": "REL",
//!             "incremental": "5891938",
//!             "release": "10",
//!             "sdk": 29
//!         }
//!     }
//! }"#;
//!
//! let fallback = random_from_uin(123456789);
//! let device = from_json(json, &fallback)?;
//! assert_eq!(device.display, "ROG Phone 3");
//! assert_eq!(device.product, "ASUS_I003DD");
//! # Ok(())
//! # }
//! ```
//!

use anyhow::{anyhow, bail, Result};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use ricq::{device::OSVersion, Device};
use ricq_core::protocol::qimei::Qimei;
use serde_json::{Map, Value};

macro_rules! parse_batch {
    ($version:ty, $json:ident, $fallback:ident, $($key:expr => $name:ident,)*) => {
        Device {
            $($name: <$version>::parse($json, $key, || $fallback.$name.clone())?,)*
        }
    };
}

macro_rules! parse {
    ($version:ty, $json:ident, $fallback:ident) => {
        parse_batch!($version, $json, $fallback,
            "display" => display,
            "product" => product,
            "device" => device,
            "board" => board,
            "model" => model,
            "fingerprint" => finger_print,
            "bootId" => boot_id,
            "procVersion" => proc_version,
            "imei" => imei,
            "brand" => brand,
            "bootloader" => bootloader,
            "baseBand" => base_band,
            "version" => version,
            "simInfo" => sim_info,
            "osType" => os_type,
            "macAddress" => mac_address,
            "ipAddress" => ip_address,
            "wifiBSSID" => wifi_bssid,
            "wifiSSID" => wifi_ssid,
            "imsiMd5" => imsi_md5,
            "androidId" => android_id,
            "apn" => apn,
            "vendorName" => vendor_name,
            "vendorOsName" => vendor_os_name,
            "qimei" => qimei,
        )
    }
}

/// 从 `device.json` 中读取设备信息。
///
/// `device.json` 采用 **mirai 的格式**，与 ricq 的直接定义不兼容。
///
/// # Arguments
///
/// * `json` - `device.json` 的内容。
/// * `fallback` - 某一项不存在时的默认值。
///
/// # Examples
///
/// ```no_run
/// # async fn _f() -> anyhow::Result<()> {
/// let json = tokio::fs::read_to_string("device.json").await?;
/// let fallback = chocho_login::device::random_from_uin(123456789);
/// let device = chocho_login::device::from_json(&json, &fallback)?;
/// println!("{:?}", device);
/// # Ok(())
/// # }
/// ```
pub fn from_json(json: &str, fallback: &Device) -> Result<Device> {
    let json: Value = serde_json::from_str(json)?;
    let json = json
        .as_object()
        .ok_or_else(|| anyhow!("根对象不是 `Object`"))?;
    // 查看版本
    let version = json
        .get("deviceInfoVersion")
        .map(|v| v.as_i64().unwrap_or(-1))
        .unwrap_or(1);
    match version {
        1 => {
            // 版本1：字符串全部使用 UTF-8 字节数组表示，MD5 使用字节数组表示
            Ok(parse!(V1, json, fallback))
        }
        2 => {
            // 版本2：字符串直接储存，MD5 使用十六进制表示
            let json = json
                .get("data")
                .and_then(|v| v.as_object())
                .ok_or_else(|| anyhow!("未找到 `data` 字段"))?;
            Ok(parse!(V2, json, fallback))
        }
        _ => bail!("未知的 `deviceInfoVersion` 值: {}", version),
    }
}

/// 以 QQ 号为种子生成随机的设备信息。
///
/// 使用 `rand_chacha` 作为随机数生成器，因此可以保证相同的 QQ 号生成的设备信息相同。
///
/// # Examples
///
/// ```
/// let device = chocho_login::device::random_from_uin(123456789);
/// assert_eq!(device.display, "RICQ.110281.001");
/// ```
pub fn random_from_uin(uin: i64) -> Device {
    let mut seed = ChaCha8Rng::seed_from_u64(uin as u64);
    Device::random_with_rng(&mut seed)
}

macro_rules! dump_batch {
    ($json:ident, $device:ident, $($key:expr => $name:ident,)*) => {
        $($json.insert($key.to_string(), V2::dump(&$device.$name));)*
    };
}

macro_rules! dump {
    ($json:ident, $device:ident) => {
        dump_batch!($json, $device,
            "display" => display,
            "product" => product,
            "device" => device,
            "board" => board,
            "model" => model,
            "fingerprint" => finger_print,
            "bootId" => boot_id,
            "procVersion" => proc_version,
            "imei" => imei,
            "brand" => brand,
            "bootloader" => bootloader,
            "baseBand" => base_band,
            "version" => version,
            "simInfo" => sim_info,
            "osType" => os_type,
            "macAddress" => mac_address,
            "ipAddress" => ip_address,
            "wifiBSSID" => wifi_bssid,
            "wifiSSID" => wifi_ssid,
            "imsiMd5" => imsi_md5,
            "androidId" => android_id,
            "apn" => apn,
            "vendorName" => vendor_name,
            "vendorOsName" => vendor_os_name,
        )
    }
}

/// 将设备信息写入 `device.json`。
pub(crate) fn to_json(device: &Device) -> Result<String> {
    let mut json = Map::new();
    json.insert("deviceInfoVersion".into(), Value::Number(2.into()));
    json.insert("data".into(), {
        let mut json = Map::new();
        dump!(json, device);
        json.into()
    });
    Ok(serde_json::to_string_pretty(&json)?)
}

trait Parse<T> {
    fn parse(json: &Map<String, Value>, key: &str, fallback: impl FnOnce() -> T) -> Result<T>;
}

struct V1;

impl Parse<String> for V1 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        fallback: impl FnOnce() -> String,
    ) -> Result<String> {
        json.get(key)
            .map(|v| -> Result<String> {
                if let Some(s) = v.as_str() {
                    return Ok(s.to_string());
                }
                let bytes = v
                    .as_array()
                    .ok_or_else(|| anyhow!("`{}` 格式错误", key))?
                    .iter()
                    .map(|b| b.as_i64())
                    .collect::<Option<Vec<i64>>>()
                    .ok_or_else(|| anyhow!("`{}` 格式错误", key))?
                    .iter()
                    .map(|b| b.to_le_bytes()[0])
                    .collect::<Vec<u8>>();
                Ok(String::from_utf8(bytes)?)
            })
            .unwrap_or_else(|| Ok(fallback()))
    }
}

impl Parse<Vec<u8>> for V1 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        fallback: impl FnOnce() -> Vec<u8>,
    ) -> Result<Vec<u8>> {
        json.get(key)
            .map(|v| -> Result<Vec<u8>> {
                let bytes = v
                    .as_array()
                    .ok_or_else(|| anyhow!("`{}` 格式错误", key))?
                    .iter()
                    .map(|b| b.as_i64())
                    .collect::<Option<Vec<i64>>>()
                    .ok_or_else(|| anyhow!("`{}` 格式错误", key))?
                    .iter()
                    .map(|b| b.to_le_bytes()[0])
                    .collect::<Vec<u8>>();
                Ok(bytes)
            })
            .unwrap_or_else(|| Ok(fallback()))
    }
}

impl Parse<u32> for V1 {
    fn parse(json: &Map<String, Value>, key: &str, fallback: impl FnOnce() -> u32) -> Result<u32> {
        json.get(key)
            .map(|v| -> Result<u32> {
                let value = v.as_i64().ok_or_else(|| anyhow!("`{}` 格式错误", key))?;
                Ok(value as u32)
            })
            .unwrap_or_else(|| Ok(fallback()))
    }
}

impl Parse<OSVersion> for V1 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        fallback: impl FnOnce() -> OSVersion,
    ) -> Result<OSVersion> {
        let version = json
            .get(key)
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow!("`{}` 格式错误", key))?;
        let fallback = fallback();
        let incremental = V1::parse(version, "incremental", || fallback.incremental)?;
        let release = V1::parse(version, "release", || fallback.release)?;
        let codename = V1::parse(version, "codename", || fallback.codename)?;
        let sdk = V1::parse(version, "sdk", || fallback.sdk)?;
        Ok(OSVersion {
            incremental,
            release,
            codename,
            sdk,
        })
    }
}

impl Parse<Option<Qimei>> for V1 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        _fallback: impl FnOnce() -> Option<Qimei>,
    ) -> Result<Option<Qimei>> {
        match json.get(key) {
            None => Ok(None),
            Some(v) => {
                let qimei = v.as_object().ok_or_else(|| anyhow!("`{}` 格式错误", key))?;
                let q16 = <V1 as Parse<String>>::parse(qimei, "q16", || "".to_string())?;
                let q36 = <V1 as Parse<String>>::parse(qimei, "q36", || "".to_string())?;
                if q16.is_empty() || q36.is_empty() {
                    Ok(None)
                } else {
                    let qimei = Qimei { q16, q36 };
                    Ok(Some(qimei))
                }
            }
        }
    }
}

struct V2;

impl Parse<String> for V2 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        fallback: impl FnOnce() -> String,
    ) -> Result<String> {
        json.get(key)
            .map(|v| -> Result<String> {
                Ok(v.as_str()
                    .ok_or_else(|| anyhow!("`{}` 格式错误", key))?
                    .to_string())
            })
            .unwrap_or_else(|| Ok(fallback()))
    }
}

impl Parse<Vec<u8>> for V2 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        fallback: impl FnOnce() -> Vec<u8>,
    ) -> Result<Vec<u8>> {
        json.get(key)
            .map(|v| -> Result<Vec<u8>> {
                let hex = v.as_str().ok_or_else(|| anyhow!("`{}` 格式错误", key))?;
                Ok(hex::decode(hex)?)
            })
            .unwrap_or_else(|| Ok(fallback()))
    }
}

impl Parse<u32> for V2 {
    fn parse(json: &Map<String, Value>, key: &str, fallback: impl FnOnce() -> u32) -> Result<u32> {
        json.get(key)
            .map(|v| -> Result<u32> {
                let value = v.as_i64().ok_or_else(|| anyhow!("`{}` 格式错误", key))?;
                Ok(value.try_into()?)
            })
            .unwrap_or_else(|| Ok(fallback()))
    }
}

impl Parse<OSVersion> for V2 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        fallback: impl FnOnce() -> OSVersion,
    ) -> Result<OSVersion> {
        let version = json
            .get(key)
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow!("`{}` 格式错误", key))?;
        let fallback = fallback();
        let incremental = V2::parse(version, "incremental", || fallback.incremental)?;
        let release = V2::parse(version, "release", || fallback.release)?;
        let codename = V2::parse(version, "codename", || fallback.codename)?;
        let sdk = V2::parse(version, "sdk", || fallback.sdk)?;
        Ok(OSVersion {
            incremental,
            release,
            codename,
            sdk,
        })
    }
}

impl Parse<Option<Qimei>> for V2 {
    fn parse(
        json: &Map<String, Value>,
        key: &str,
        _fallback: impl FnOnce() -> Option<Qimei>,
    ) -> Result<Option<Qimei>> {
        match json.get(key) {
            None => Ok(None),
            Some(v) => {
                let qimei = v.as_object().ok_or_else(|| anyhow!("`{}` 格式错误", key))?;
                let q16 = <V2 as Parse<String>>::parse(qimei, "q16", || "".to_string())?;
                let q36 = <V2 as Parse<String>>::parse(qimei, "q36", || "".to_string())?;
                if q16.is_empty() || q36.is_empty() {
                    Ok(None)
                } else {
                    let qimei = Qimei { q16, q36 };
                    Ok(Some(qimei))
                }
            }
        }
    }
}

trait Dump<T> {
    fn dump(value: &T) -> Value;
}

impl Dump<String> for V2 {
    fn dump(value: &String) -> Value {
        value.to_string().into()
    }
}

impl Dump<Vec<u8>> for V2 {
    fn dump(value: &Vec<u8>) -> Value {
        hex::encode(value).into()
    }
}

impl Dump<u32> for V2 {
    fn dump(value: &u32) -> Value {
        (*value as u64).into()
    }
}

impl Dump<OSVersion> for V2 {
    fn dump(value: &OSVersion) -> Value {
        let mut map = Map::new();
        map.insert("incremental".to_string(), V2::dump(&value.incremental));
        map.insert("release".to_string(), V2::dump(&value.release));
        map.insert("codename".to_string(), V2::dump(&value.codename));
        map.insert("sdk".to_string(), V2::dump(&value.sdk));
        map.into()
    }
}

impl Dump<Option<Qimei>> for V2 {
    fn dump(value: &Option<Qimei>) -> Value {
        match value {
            None => Value::Null,
            Some(qimei) => {
                let mut map = Map::new();
                map.insert("q16".to_string(), V2::dump(&qimei.q16));
                map.insert("q36".to_string(), V2::dump(&qimei.q36));
                map.into()
            }
        }
    }
}
