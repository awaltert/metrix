extern crate exponential_decay_histogram;
#[macro_use]
extern crate json;
extern crate metrics;

use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use processor::TelemetryMessage;
use instruments::{Cockpit, HandlesObservations, Panel};

pub mod instruments;
pub mod snapshot;
pub mod processor;
pub mod driver;

/// An observation that has been made
#[derive(Clone)]
pub enum Observation<L> {
    /// Observed many occurances at th given timestamp
    Observed {
        label: L,
        count: u64,
        timestamp: Instant,
    },
    /// Observed one occurrence at the given timestamp
    ObservedOne { label: L, timestamp: Instant },
    /// Observed one occurence with a value at a given timestamp.
    ObservedOneValue {
        label: L,
        value: u64,
        timestamp: Instant,
    },
}

impl<L> Observation<L> where {
    pub fn label(&self) -> &L {
        match *self {
            Observation::Observed { ref label, .. } => label,
            Observation::ObservedOne { ref label, .. } => label,
            Observation::ObservedOneValue { ref label, .. } => label,
        }
    }
}

pub trait TransmitsTelemetryData<L> {
    /// Collect an observation.
    fn transmit(&self, observation: Observation<L>);

    /// Observed `count` occurences at time `timestamp`
    ///
    /// Convinience method. Simply calls `transmit`
    fn observed(&self, label: L, count: u64, timestamp: Instant) {
        self.transmit(Observation::Observed {
            label,
            count,
            timestamp,
        })
    }

    /// Observed one occurence at time `timestamp`
    ///
    /// Convinience method. Simply calls `transmit`
    fn observed_one(&self, label: L, timestamp: Instant) {
        self.transmit(Observation::ObservedOne { label, timestamp })
    }

    /// Observed one occurence with value `value` at time `timestamp`
    ///
    /// Convinience method. Simply calls `transmit`
    fn observed_one_value(&self, label: L, value: u64, timestamp: Instant) {
        self.transmit(Observation::ObservedOneValue {
            label,
            value,
            timestamp,
        })
    }

    fn observed_duration(&self, label: L, duration: Duration, timestamp: Instant) {
        let nanos = (duration.as_secs() * 1_000_000_000) + (duration.subsec_nanos() as u64);
        self.observed_one_value(label, nanos, timestamp)
    }

    fn measure_time(&self, label: L, from: Instant) {
        let now = Instant::now();
        if from <= now {
            self.observed_duration(label, now - from, now)
        }
    }

    /// Observed `count` occurences at now.
    ///
    /// Convinience method. Simply calls `transmit`
    fn observed_now(&self, label: L, count: u64) {
        self.observed(label, count, Instant::now())
    }

    /// Observed one occurence now
    ///
    /// Convinience method. Simply calls `transmit`
    fn observed_one_now(&self, label: L) {
        self.observed_one(label, Instant::now())
    }

    /// Observed one occurence with value `value` now
    ///
    /// Convinience method. Simply calls `transmit`
    fn observed_one_value_now(&self, label: L, value: u64) {
        self.observed_one_value(label, value, Instant::now())
    }

    fn observed_one_duration_now(&self, label: L, duration: Duration) {
        self.observed_duration(label, duration, Instant::now());
    }

    fn add_handler(&self, handler: Box<HandlesObservations<Label = L>>);
    fn add_cockpit(&self, cockpit: Cockpit<L>);
    fn add_panel_to_cockpit(&self, cockpit_name: String, label: L, panel: Panel);
}

/// Transmits `Observation`s to the backend
#[derive(Clone)]
pub struct TelemetryTransmitter<L> {
    sender: mpsc::Sender<TelemetryMessage<L>>,
}

impl<L> TelemetryTransmitter<L>
where
    L: Send + 'static,
{
    pub fn synced(&self) -> TelemetryTransmitterSync<L> {
        TelemetryTransmitterSync {
            sender: Arc::new(Mutex::new(self.sender.clone())),
        }
    }
}

impl<L> TransmitsTelemetryData<L> for TelemetryTransmitter<L> {
    fn transmit(&self, observation: Observation<L>) {
        if let Err(_err) = self.sender.send(TelemetryMessage::Observation(observation)) {
            // maybe log...
        }
    }

    fn add_handler(&self, handler: Box<HandlesObservations<Label = L>>) {
        if let Err(_err) = self.sender.send(TelemetryMessage::AddHandler(handler)) {
            // maybe log...
        }
    }

    fn add_cockpit(&self, cockpit: Cockpit<L>) {
        if let Err(_err) = self.sender.send(TelemetryMessage::AddCockpit(cockpit)) {
            // maybe log...
        }
    }

    fn add_panel_to_cockpit(&self, cockpit_name: String, label: L, panel: Panel) {
        if let Err(_err) = self.sender.send(TelemetryMessage::AddPanel {
            cockpit_name,
            label,
            panel,
        }) {
            // maybe log...
        }
    }
}

/// Transmits `Observation`s to the backend and has the `Sync` marker
///
/// This is almost the same as the `TelemetryTransmitter`.
///
/// Since a `Sender` for a channel is not `Sync` this
/// struct wraps the `Sender` in an `Arc<Mutex<_>>` so that
/// it can be shared between threads.
#[derive(Clone)]
pub struct TelemetryTransmitterSync<L> {
    sender: Arc<Mutex<mpsc::Sender<TelemetryMessage<L>>>>,
}

impl<L> TelemetryTransmitterSync<L>
where
    L: Send + 'static,
{
}

impl<L> TransmitsTelemetryData<L> for TelemetryTransmitterSync<L> {
    fn transmit(&self, observation: Observation<L>) {
        if let Err(_err) = self.sender
            .lock()
            .unwrap()
            .send(TelemetryMessage::Observation(observation))
        {
            // maybe log...
        }
    }

    fn add_handler(&self, handler: Box<HandlesObservations<Label = L>>) {
        if let Err(_err) = self.sender
            .lock()
            .unwrap()
            .send(TelemetryMessage::AddHandler(handler))
        {
            // maybe log...
        }
    }

    fn add_cockpit(&self, cockpit: Cockpit<L>) {
        if let Err(_err) = self.sender
            .lock()
            .unwrap()
            .send(TelemetryMessage::AddCockpit(cockpit))
        {
            // maybe log...
        }
    }

    fn add_panel_to_cockpit(&self, cockpit_name: String, label: L, panel: Panel) {
        if let Err(_err) = self.sender
            .lock()
            .unwrap()
            .send(TelemetryMessage::AddPanel {
                cockpit_name,
                label,
                panel,
            }) {
            // maybe log...
        }
    }
}
