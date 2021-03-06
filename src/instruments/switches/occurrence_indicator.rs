use std::time::{Duration, Instant};

use instruments::{Instrument, Update, Updates};
use snapshot::Snapshot;
use {Descriptive, PutsSnapshot};
use util;

/// Changes the state based on the occurrence of an observation
/// within a given time.
///
/// Can be used for alerting, e.g. if something
/// bad was observed within a given timeframe.
pub struct OccurrenceIndicator {
    name: String,
    title: Option<String>,
    description: Option<String>,
    if_happened_within: Duration,
    happened_last: Instant,
    invert: bool,
}

impl OccurrenceIndicator {
    pub fn new_with_defaults<T: Into<String>>(name: T) -> OccurrenceIndicator {
        OccurrenceIndicator {
            name: name.into(),
            title: None,
            description: None,
            if_happened_within: Duration::from_secs(60),
            happened_last: Instant::now() - Duration::from_secs(60),
            invert: false,
        }
    }

    /// Gets the name of this `OccurenceIndicator`
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the name if this `OccurenceIndicator`.
    ///
    /// The name is a path segment within a `Snapshot`
    pub fn set_name<T: Into<String>>(&mut self, name: T) {
        self.name = name.into();
    }

    /// Sets the `title` of this `OccurenceIndicator`.
    ///
    /// A title can be part of a descriptive `Snapshot`
    pub fn set_title<T: Into<String>>(&mut self, title: T) {
        self.title = Some(title.into())
    }

    /// Sets the `description` of this `OccurenceIndicator`.
    ///
    /// A description can be part of a descriptive `Snapshot`
    pub fn set_description<T: Into<String>>(&mut self, description: T) {
        self.description = Some(description.into())
    }

    /// Set whether the current value should be inverted in a snapshot or not
    ///
    /// Default is `false`
    pub fn set_invert(&mut self, invert: bool) {
        self.invert = invert
    }

    /// The current value should be inverted in a snapshot
    ///
    /// Same as `self.set_invert(true);`
    pub fn enable_invert(&mut self) {
        self.invert = true
    }

    /// return whether invert is on or off
    pub fn invert(&self) -> bool {
        self.invert
    }

    pub fn set_if_happened_within(&mut self, d: Duration) {
        self.if_happened_within = d;
        self.happened_last = Instant::now() - d;
    }

    pub fn if_happened_within(&self) -> Duration {
        self.if_happened_within
    }

    /// Returns the current state
    pub fn state(&self) -> bool {
        let current_state = self.happened_last + self.if_happened_within >= Instant::now();

        if self.invert {
            !current_state
        } else {
            current_state
        }
    }
}

impl Instrument for OccurrenceIndicator {}

impl PutsSnapshot for OccurrenceIndicator {
    fn put_snapshot(&self, into: &mut Snapshot, descriptive: bool) {
        util::put_postfixed_descriptives(self, &self.name, into, descriptive);

        into.items.push((self.name.clone(), self.state().into()));
    }
}

impl Updates for OccurrenceIndicator {
    fn update(&mut self, _: &Update) {
        self.happened_last = Instant::now()
    }
}

impl Descriptive for OccurrenceIndicator {
    fn title(&self) -> Option<&str> {
        self.title.as_ref().map(|n| &**n)
    }

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(|n| &**n)
    }
}
