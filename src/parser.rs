use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use toml::Value as TomlValue;

#[derive(Debug, Clone)]
pub enum DataValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<DataValue>),
    Object(HashMap<String, DataValue>),
    Null,
}

impl DataValue {
    pub fn to_display_string(&self) -> String {
        match self {
            DataValue::String(s) => s.clone(),
            DataValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            DataValue::Boolean(b) => b.to_string(),
            DataValue::Array(_) => "▸ table".to_string(),
            DataValue::Object(_) => "▸ table".to_string(),
            DataValue::Null => "null".to_string(),
        }
    }

    pub fn is_simple_value(&self) -> bool {
        matches!(self, DataValue::String(_) | DataValue::Number(_) | DataValue::Boolean(_) | DataValue::Null)
    }
}

#[derive(Debug)]
pub struct ParsedData {
    pub data: DataValue,
    pub format: String,
}

pub fn parse_file(file_path: &str) -> Result<ParsedData, Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    let content = fs::read_to_string(path)?;
    
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "json" => {
            let json_value: JsonValue = serde_json::from_str(&content)?;
            Ok(ParsedData {
                data: json_to_data_value(json_value),
                format: "JSON".to_string(),
            })
        }
        "yaml" | "yml" => {
            let yaml_value: YamlValue = serde_yaml::from_str(&content)?;
            Ok(ParsedData {
                data: yaml_to_data_value(yaml_value),
                format: "YAML".to_string(),
            })
        }
        "toml" => {
            let toml_value: TomlValue = toml::from_str(&content)?;
            Ok(ParsedData {
                data: toml_to_data_value(toml_value),
                format: "TOML".to_string(),
            })
        }
        _ => Err(format!("Unsupported file format: {}", extension).into()),
    }
}

fn json_to_data_value(value: JsonValue) -> DataValue {
    match value {
        JsonValue::String(s) => DataValue::String(s),
        JsonValue::Number(n) => DataValue::Number(n.as_f64().unwrap_or(0.0)),
        JsonValue::Bool(b) => DataValue::Boolean(b),
        JsonValue::Array(arr) => {
            DataValue::Array(arr.into_iter().map(json_to_data_value).collect())
        }
        JsonValue::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_data_value(v));
            }
            DataValue::Object(map)
        }
        JsonValue::Null => DataValue::Null,
    }
}

fn yaml_to_data_value(value: YamlValue) -> DataValue {
    match value {
        YamlValue::String(s) => DataValue::String(s),
        YamlValue::Number(n) => DataValue::Number(n.as_f64().unwrap_or(0.0)),
        YamlValue::Bool(b) => DataValue::Boolean(b),
        YamlValue::Sequence(arr) => {
            DataValue::Array(arr.into_iter().map(yaml_to_data_value).collect())
        }
        YamlValue::Mapping(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                if let YamlValue::String(key) = k {
                    map.insert(key, yaml_to_data_value(v));
                } else {
                    map.insert(format!("{:?}", k), yaml_to_data_value(v));
                }
            }
            DataValue::Object(map)
        }
        YamlValue::Null => DataValue::Null,
        _ => DataValue::String(format!("{:?}", value)),
    }
}

fn toml_to_data_value(value: TomlValue) -> DataValue {
    match value {
        TomlValue::String(s) => DataValue::String(s),
        TomlValue::Integer(n) => DataValue::Number(n as f64),
        TomlValue::Float(n) => DataValue::Number(n),
        TomlValue::Boolean(b) => DataValue::Boolean(b),
        TomlValue::Array(arr) => {
            DataValue::Array(arr.into_iter().map(toml_to_data_value).collect())
        }
        TomlValue::Table(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k, toml_to_data_value(v));
            }
            DataValue::Object(map)
        }
        TomlValue::Datetime(dt) => DataValue::String(dt.to_string()),
    }
}

