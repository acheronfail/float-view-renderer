use sailfish::TemplateSimple;

use crate::input::DataPoint;

fn format_float(value: f32, as_int: bool) -> String {
    if as_int {
        value.round().to_string()
    } else {
        format!("{:.2}", value)
    }
}

#[derive(Debug, Default)]
struct Speedometer {
    title: String,
    min: f32,
    max: f32,
    value: f32,
    units: String,
    step: Option<f32>,
    title_color: Option<String>,
    mini_tick_color: Option<String>,
    main_tick_color: Option<String>,
    arc_color: Option<String>,
    needle_color: Option<String>,
    tick_label_color: Option<String>,
    redline_threshold_pct: Option<f32>,
    format_as_float: Option<bool>,
}

#[derive(Debug)]
enum Item {
    Title(String),
    Datum {
        label: String,
        value: String,
        color: Option<String>,
    },
}

#[derive(Debug, TemplateSimple)]
#[template(path = "view.stpl")]
struct SvgTemplate {
    speedometers: Vec<Speedometer>,
    items: Vec<Item>,
}

pub fn render_svg(data_point: &DataPoint, cell_count: &Option<u8>) -> String {
    let voltage_color = "#34d399";
    let current_color = "#67e8f9";
    let speed_color = "#fde68a";
    let duty_color = "#f472b6";
    let temp_color = "#fbbf24";

    let speedometers = vec![
        Speedometer {
            title: "Speed".to_string(),
            step: Some(10.0),
            min: 0.0,
            max: 50.0,
            value: data_point.speed,
            units: " km/h".to_string(),
            format_as_float: Some(true),
            title_color: Some(speed_color.to_string()),
            needle_color: Some(speed_color.to_string()),
            ..Default::default()
        },
        Speedometer {
            title: "Duty Cycle".to_string(),
            step: Some(20.0),
            min: 0.0,
            max: 100.0,
            value: data_point.duty_cycle,
            units: "%".to_string(),
            title_color: Some(duty_color.to_string()),
            needle_color: Some(duty_color.to_string()),
            ..Default::default()
        },
    ];

    // motor

    let mut items = vec![
        Item::Title("Motor".to_string()),
        Item::Datum {
            label: "Current".to_string(),
            value: format!("{} A", format_float(data_point.motor_current, false)),
            color: Some(current_color.to_string()),
        },
    ];

    if let Some(field_weakening) = data_point.field_weakening {
        items.push(Item::Datum {
            label: "Field Weakening".to_string(),
            value: format!("{} A", format_float(field_weakening, false)),
            color: Some(current_color.to_string()),
        });
    }

    // temps

    items.push(Item::Title("Temperatures".to_string()));
    items.append(&mut vec![
        Item::Datum {
            label: "Motor".to_string(),
            value: format!("{} °C", format_float(data_point.temp_motor, false)),
            color: Some(temp_color.to_string()),
        },
        Item::Datum {
            label: "Controller".to_string(),
            value: format!("{} °C", format_float(data_point.temp_mosfet, false)),
            color: Some(temp_color.to_string()),
        },
    ]);
    if let Some(temp_battery) = data_point.temp_battery {
        items.push(Item::Datum {
            label: "Battery".to_string(),
            value: format!("{} °C", format_float(temp_battery, false)),
            color: Some(temp_color.to_string()),
        });
    }

    // battery

    items.push(Item::Title("Battery".to_string()));
    if let Some(cell_count) = cell_count {
        items.push(Item::Datum {
            label: "Voltage (per cell)".to_string(),
            value: format!(
                "{} V",
                format_float(data_point.batt_voltage / *cell_count as f32, false)
            ),
            color: Some(voltage_color.to_string()),
        });
    }
    items.append(&mut vec![
        Item::Datum {
            label: "Voltage".to_string(),
            value: format!("{} V", format_float(data_point.batt_voltage, false)),
            color: Some(voltage_color.to_string()),
        },
        Item::Datum {
            label: "Current".to_string(),
            value: format!("{} A", format_float(data_point.batt_current, false)),
            color: Some(current_color.to_string()),
        },
        Item::Datum {
            label: "Watts".to_string(),
            value: format!(
                "{} W",
                (data_point.batt_voltage * data_point.batt_current).round() as usize
            ),
            color: None,
        },
    ]);

    SvgTemplate {
        speedometers,
        items,
    }
    .render_once()
    .unwrap()
}
