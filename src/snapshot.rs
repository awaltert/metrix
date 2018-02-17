use json::{stringify, stringify_pretty, JsonValue};

pub struct JsonConfig {
    /// Serialize `true` as `1` and `false` as `0`
    pub make_booleans_ints: bool,

    /// Configure pretty JSON output.
    ///
    /// Produce pretty JSON with the given indentation if `Some(indentation)`.
    /// If `None` compact JSON is generated.
    pub pretty: Option<u16>,
}

impl Default for JsonConfig {
    fn default() -> JsonConfig {
        JsonConfig {
            make_booleans_ints: false,
            pretty: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub items: Vec<(String, ItemKind)>,
}

impl Default for Snapshot {
    fn default() -> Snapshot {
        Snapshot { items: Vec::new() }
    }
}

impl Snapshot {
    pub fn push<K: Into<String>>(&mut self, k: K, v: ItemKind) {
        self.items.push((k.into(), v))
    }

    /// Output JSON with default settings.
    pub fn to_default_json(&self) -> String {
        self.to_json_internal(&JsonConfig::default())
    }

    /// Output JSON with the given settings.
    pub fn to_json(&self, config: &JsonConfig) -> String {
        self.to_json_internal(config)
    }

    fn to_json_internal(&self, config: &JsonConfig) -> String {
        let data = self.to_json_value(config);

        if let Some(indent) = config.pretty {
            stringify_pretty(data, indent)
        } else {
            stringify(data)
        }
    }

    fn to_json_value(&self, config: &JsonConfig) -> JsonValue {
        let mut data = JsonValue::new_object();

        self.items
            .iter()
            .for_each(|&(ref k, ref v)| data[k] = v.to_json_value(config));

        data
    }
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    Text(String),
    Boolean(bool),
    Float(f64),
    UInt(u64),
    Int(i64),
    Snapshot(Snapshot),
}

impl ItemKind {
    fn to_json_value(&self, config: &JsonConfig) -> JsonValue {
        match *self {
            ItemKind::Text(ref v) => v.clone().into(),
            ItemKind::Boolean(v) => if config.make_booleans_ints {
                if v {
                    1.into()
                } else {
                    0.into()
                }
            } else {
                v.into()
            },
            ItemKind::Float(v) => v.into(),
            ItemKind::UInt(v) => v.into(),
            ItemKind::Int(v) => v.into(),
            ItemKind::Snapshot(ref snapshot) => snapshot.to_json_value(config),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeterSnapshot {
    pub one_minute: MeterRate,
    pub five_minutes: MeterRate,
    pub fifteen_minutes: MeterRate,
}

impl MeterSnapshot {
    pub fn put_snapshot(&self, into: &mut Snapshot, descriptive: bool) {
        let mut one_minute = Snapshot::default();
        self.one_minute.put_snapshot(&mut one_minute, descriptive);
        into.items
            .push(("one_minute".to_string(), ItemKind::Snapshot(one_minute)));
        let mut five_minutes = Snapshot::default();
        self.five_minutes
            .put_snapshot(&mut five_minutes, descriptive);
        into.items
            .push(("five_minutes".to_string(), ItemKind::Snapshot(five_minutes)));
        let mut fifteen_minutes = Snapshot::default();
        self.fifteen_minutes
            .put_snapshot(&mut fifteen_minutes, descriptive);
        into.items.push((
            "fifteen_minutes".to_string(),
            ItemKind::Snapshot(fifteen_minutes),
        ));
    }
}

#[derive(Debug, Clone)]
pub struct MeterRate {
    pub rate: f64,
    pub count: u64,
}

impl MeterRate {
    fn put_snapshot(&self, into: &mut Snapshot, _descriptive: bool) {
        into.items
            .push(("rate".to_string(), ItemKind::Float(self.rate)));
        into.items
            .push(("count".to_string(), ItemKind::UInt(self.count)));
    }
}

#[derive(Debug, Clone)]
pub struct HistogramSnapshot {
    pub max: i64,
    pub min: i64,
    pub mean: f64,
    pub stddev: f64,
    pub count: u64,
    pub quantiles: Vec<(u16, i64)>,
}

impl HistogramSnapshot {
    pub fn put_snapshot(&self, into: &mut Snapshot, _descriptive: bool) {
        into.items
            .push(("max".to_string(), ItemKind::Int(self.max)));
        into.items
            .push(("min".to_string(), ItemKind::Int(self.min)));
        into.items
            .push(("mean".to_string(), ItemKind::Float(self.mean)));
        into.items
            .push(("stddev".to_string(), ItemKind::Float(self.stddev)));
        into.items
            .push(("count".to_string(), ItemKind::UInt(self.count)));

        let mut quantiles = Snapshot::default();

        for &(ref q, ref v) in &self.quantiles {
            quantiles.items.push((format!("p{}", q), ItemKind::Int(*v)));
        }

        into.items
            .push(("quantiles".to_string(), ItemKind::Snapshot(quantiles)));
    }
}
