use std::f32::NAN;
use std::fs::File;
use std::io::{BufReader, Read};

use anyhow::{bail, Result};
use serde_derive::Deserialize;

pub struct DataPoint {
    pub duration: f32,

    pub speed: f32,
    pub duty_cycle: f32,

    pub motor_current: f32,
    pub field_weakening: Option<f32>,

    pub temp_motor: f32,
    pub temp_mosfet: f32,
    pub temp_battery: Option<f32>,

    pub batt_voltage: f32,
    pub batt_current: f32,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct FloatControlCsv {
    #[serde(rename = "Time(s)")]
    time_seconds: f32,
    #[serde(rename = "State")]
    state: String,
    #[serde(rename = "Distance(km)")]
    distance_km: Option<f32>,
    #[serde(rename = "Distance(mi)")]
    distance_mi: Option<f32>,
    #[serde(rename = "Speed(km/h)")]
    speed_kmh: Option<f32>,
    #[serde(rename = "Speed(mph)")]
    speed_mph: Option<f32>,
    #[serde(rename = "Duty%")]
    duty_cycle: String,
    #[serde(rename = "Voltage")]
    voltage: f32,
    #[serde(rename = "I-Battery")]
    current_battery: f32,
    #[serde(rename = "I-Motor")]
    current_motor: f32,
    #[serde(rename = "I-FldWeak")]
    current_field_weakening: Option<f32>,
    #[serde(rename = "Requested Amps")]
    current_requested: f32,
    #[serde(rename = "I-Booster")]
    current_booster: f32,
    #[serde(rename = "Altitude(m)")]
    altitude: f32,
    #[serde(rename = "GPS-Lat")]
    gps_lat: f32,
    #[serde(rename = "GPS-Long")]
    gps_lon: f32,
    #[serde(rename = "GPS-Accuracy")]
    gps_acc: f32,
    #[serde(rename = "True Pitch")]
    true_pitch: f32,
    #[serde(rename = "Pitch")]
    pitch: f32,
    #[serde(rename = "Roll")]
    roll: f32,
    #[serde(rename = "Setpoint")]
    setpoint: f32,
    #[serde(rename = "SP-ATR")]
    setpoint_atr: f32,
    #[serde(rename = "SP-Carve")]
    setpoint_carve: f32,
    #[serde(rename = "SP-TrqTlt")]
    setpoint_torque_tilt: f32,
    #[serde(rename = "SP-BrkTlt")]
    setpoint_break_tilt: f32,
    #[serde(rename = "SP-Remote")]
    setpoint_remote: f32,
    #[serde(rename = "T-Mosfet")]
    temp_mosfet: f32,
    #[serde(rename = "T-Mot")]
    temp_motor: f32,
    #[serde(rename = "T-Batt")]
    temp_battery: f32,
    #[serde(rename = "T-BMS")]
    temp_bms: Option<f32>,
    #[serde(rename = "T-Battery")]
    temp_bms_battery: Option<f32>,
    #[serde(rename = "BMS-Fault")]
    bms_fault: Option<u8>,
    #[serde(rename = "ADC1")]
    adc1: f32,
    #[serde(rename = "ADC2")]
    acd2: f32,
    #[serde(rename = "Motor-Fault")]
    fault_motor: u8,
    #[serde(rename = "Ah")]
    amp_hours: f32,
    #[serde(rename = "Ah Charged")]
    amp_hours_charged: f32,
    #[serde(rename = "Wh")]
    wh: f32,
    #[serde(rename = "Wh Charged")]
    wh_charged: f32,
    #[serde(rename = "ERPM")]
    erpm: u32,
}

impl FloatControlCsv {
    fn speed_kmh(&self) -> f32 {
        self.speed_kmh
            .or(self.speed_mph.map(|mph| mph * 1.60934))
            .unwrap_or(NAN)
    }

    fn to_data_point(&self, prev_time: f32) -> DataPoint {
        DataPoint {
            duration: self.time_seconds - prev_time,
            speed: self.speed_kmh(),
            duty_cycle: self
                .duty_cycle
                .trim_end_matches('%')
                .parse::<f32>()
                .unwrap_or(NAN),
            motor_current: self.current_motor,
            field_weakening: self.current_field_weakening,
            temp_motor: self.temp_motor,
            temp_mosfet: self.temp_mosfet,
            temp_battery: Some(self.temp_battery),
            batt_voltage: self.voltage,
            batt_current: self.current_battery,
        }
    }
}

fn parse_float_control<R: Read>(rdr: R) -> Result<Vec<DataPoint>> {
    let mut data: Vec<DataPoint> = vec![];

    let mut rdr = csv::Reader::from_reader(rdr);
    for result in rdr.deserialize() {
        let record: FloatControlCsv = result?;
        data.push(record.to_data_point(data.last().map(|dp| dp.duration).unwrap_or(0.0)));
    }

    let has_battery_temps = data
        .iter()
        .any(|dp| dp.temp_battery.map_or(false, |temp| temp != 0.0));
    if !has_battery_temps {
        for dp in data.iter_mut() {
            dp.temp_battery = None;
        }
    }

    Ok(data)
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct FloatyLog {
    #[serde(rename = "timestamp")]
    timestamp: u64,
    #[serde(rename = "speed")]
    speed: Option<f64>,
    #[serde(rename = "dutyCycle")]
    duty_cycle: Option<f64>,
    #[serde(rename = "batteryVolts")]
    battery_volts: Option<f64>,
    #[serde(rename = "batteryPercent")]
    battery_percent: f64,
    #[serde(rename = "batteryCurrent")]
    battery_current: Option<f64>,
    #[serde(rename = "motorCurrent")]
    motor_current: Option<f64>,
    #[serde(rename = "motorTemp")]
    motor_temp: f64,
    #[serde(rename = "controllerTemp")]
    controller_temp: f64,
    #[serde(rename = "tripDistance")]
    trip_distance: f64,
    #[serde(rename = "lifeDistance")]
    life_distance: f64,
    #[serde(rename = "remainingDistance")]
    remaining_distance: f64,
    #[serde(rename = "rollAngle")]
    roll_angle: f64,
    #[serde(rename = "pitchAngle")]
    pitch_angle: f64,
    #[serde(rename = "truePitchAngle")]
    true_pitch_angle: f64,
    #[serde(rename = "inputTilt")]
    input_tilt: f64,
    #[serde(rename = "throttle")]
    throttle: f64,
    #[serde(rename = "ampHours")]
    amp_hours: f64,
    #[serde(rename = "wattHours")]
    watt_hours: f64,
    #[serde(rename = "state")]
    state: f64,
    #[serde(rename = "switchState")]
    switch_state: f64,
    #[serde(rename = "setpointAdjustmentType")]
    setpoint_adjustment_type: f64,
    #[serde(rename = "faultCode")]
    fault_code: f64,
    #[serde(rename = "adc1")]
    adc1: f64,
    #[serde(rename = "adc2")]
    adc2: f64,
}

impl FloatyLog {
    fn to_data_point(&self, start_time: u64) -> DataPoint {
        DataPoint {
            duration: (self.timestamp - start_time) as f32 / 1000.0,
            speed: self.speed.unwrap_or(f64::NAN) as f32,
            duty_cycle: self.duty_cycle.unwrap_or(f64::NAN) as f32,
            motor_current: self.motor_current.unwrap_or(f64::NAN) as f32,
            field_weakening: None,
            temp_motor: self.motor_temp as f32,
            temp_mosfet: self.controller_temp as f32,
            temp_battery: None,
            batt_voltage: self.battery_volts.unwrap_or(f64::NAN) as f32,
            batt_current: self.battery_current.unwrap_or(f64::NAN) as f32,
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct FloatyJson {
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "name")]
    name: Option<String>,
    #[serde(rename = "boardId")]
    board_id: Option<String>,
    #[serde(rename = "startTime")]
    start_time: u64,
    #[serde(rename = "endTime")]
    end_time: u64,
    #[serde(rename = "stopReason")]
    stop_reason: usize,
    #[serde(rename = "distance")]
    distance: f64,
    #[serde(rename = "logs")]
    logs: Vec<FloatyLog>,
}

fn parse_floaty<R: Read>(rdr: R) -> Result<Vec<DataPoint>> {
    let mut data: Vec<DataPoint> = vec![];

    let json: FloatyJson = serde_json::from_reader(rdr)?;

    for log in json.logs {
        data.push(log.to_data_point(json.start_time));
    }

    Ok(data)
}

pub fn parse(input_file: impl AsRef<str>) -> Result<Vec<DataPoint>> {
    let input_file = input_file.as_ref();

    let rdr = BufReader::new(File::open(&input_file)?);
    if input_file.ends_with(".zip") {
        let mut archive = zip::ZipArchive::new(rdr)?;
        let file = match archive.by_index(0) {
            Ok(file) if file.name().ends_with(".csv") => file,
            Ok(_) | Err(..) => {
                bail!("failed to find inner CSV file")
            }
        };

        return parse_float_control(file);
    }

    if input_file.ends_with(".csv") {
        return parse_float_control(rdr);
    }

    if input_file.ends_with(".json") {
        return parse_floaty(rdr);
    }

    bail!("Unsupported file format");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fc_imperial() {
        let data = parse("test_data/fc_imperial.csv").unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].speed, 16.0934);
    }

    #[test]
    fn fc_imperial_bms() {
        let data = parse("test_data/fc_imperial_bms.csv").unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].speed, 16.0934);
    }

    #[test]
    fn fc_metric() {
        let data = parse("test_data/fc_metric.csv").unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].speed, 10.0);
    }

    #[test]
    fn fc_metric_bms() {
        let data = parse("test_data/fc_metric_bms.csv").unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].speed, 10.0);
    }

    #[test]
    fn fc_metric_zip() {
        let data = parse("test_data/fc_metric.csv.zip").unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].speed, 10.0);
    }

    #[test]
    fn floaty_json() {
        let data = parse("test_data/floaty.json").unwrap();
        assert_eq!(data.len(), 3);
    }
}
