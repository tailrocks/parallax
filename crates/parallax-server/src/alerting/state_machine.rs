//! Pure alert evaluation state machine (plan 167 step 1).
//!
//! `(rule, prev_state, measurement, now) -> (next_state, transition)` with
//! consecutive-breach / consecutive-healthy hysteresis, min sample count,
//! no-data skip vs zero, between/not_between comparators, and renotify
//! interval. Exhaustive unit tests live below.

/// How a missing measurement is treated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NoDataBehavior {
    /// Leave counters untouched; no transition.
    Skip,
    /// Treat as value `0.0` with a synthetic sample count of 1.
    Zero,
}

/// Threshold comparator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AlertComparator {
    Gt,
    Gte,
    Lt,
    Lte,
    Between,
    NotBetween,
}

/// Rule severity label (display/delivery only at this layer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AlertSeverity {
    Warning,
    Critical,
}

/// Static rule parameters that drive evaluation (no IDs/storage).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RuleEvalConfig {
    pub comparator: AlertComparator,
    pub threshold: f64,
    pub threshold_upper: Option<f64>,
    pub consecutive_breaches_required: u32,
    pub consecutive_healthy_required: u32,
    pub minimum_sample_count: u64,
    pub no_data_behavior: NoDataBehavior,
    pub renotify_interval_minutes: u32,
    pub severity: AlertSeverity,
}

impl Default for RuleEvalConfig {
    fn default() -> Self {
        Self {
            comparator: AlertComparator::Gt,
            threshold: 0.0,
            threshold_upper: None,
            consecutive_breaches_required: 2,
            consecutive_healthy_required: 2,
            minimum_sample_count: 1,
            no_data_behavior: NoDataBehavior::Skip,
            renotify_interval_minutes: 30,
            severity: AlertSeverity::Warning,
        }
    }
}

/// Per-(rule, group) rolling evaluation state.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct RuleEvalState {
    pub consecutive_breaches: u32,
    pub consecutive_healthy: u32,
    pub incident_open: bool,
    /// Unix seconds of last notification (triggered or renotify).
    pub last_notified_at: Option<i64>,
    pub last_value: Option<f64>,
    pub last_sample_count: u64,
}

/// One measurement window result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AlertMeasurement {
    /// `None` = no data in window.
    pub value: Option<f64>,
    pub sample_count: u64,
}

/// Incident-facing transition emitted by one evaluation tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AlertTransition {
    None,
    OpenIncident,
    ResolveIncident,
    Renotify,
}

/// Full outcome of one evaluation tick.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EvaluationOutcome {
    pub state: RuleEvalState,
    pub transition: AlertTransition,
    /// Whether the measurement itself was a breach (after no-data handling).
    pub is_breach: bool,
    /// Effective value used (after no-data zeroing); `None` when skipped.
    pub effective_value: Option<f64>,
}

/// Apply the comparator to a concrete value.
#[must_use]
pub(super) fn value_breaches(config: &RuleEvalConfig, value: f64) -> bool {
    match config.comparator {
        AlertComparator::Gt => value > config.threshold,
        AlertComparator::Gte => value >= config.threshold,
        AlertComparator::Lt => value < config.threshold,
        AlertComparator::Lte => value <= config.threshold,
        AlertComparator::Between => {
            let upper = config.threshold_upper.unwrap_or(config.threshold);
            value >= config.threshold && value <= upper
        }
        AlertComparator::NotBetween => {
            let upper = config.threshold_upper.unwrap_or(config.threshold);
            value < config.threshold || value > upper
        }
    }
}

/// Evaluate one rule tick. Pure — no I/O.
#[must_use]
pub(crate) fn evaluate_rule(
    config: &RuleEvalConfig,
    prev: &RuleEvalState,
    measurement: AlertMeasurement,
    now_unix_secs: i64,
) -> EvaluationOutcome {
    let breaches_required = config.consecutive_breaches_required.max(1);
    let healthy_required = config.consecutive_healthy_required.max(1);

    // Resolve no-data.
    let (effective_value, sample_count, skipped) = match measurement.value {
        None => match config.no_data_behavior {
            NoDataBehavior::Skip => {
                let mut state = prev.clone();
                state.last_sample_count = measurement.sample_count;
                return EvaluationOutcome {
                    state,
                    transition: AlertTransition::None,
                    is_breach: false,
                    effective_value: None,
                };
            }
            NoDataBehavior::Zero => (0.0_f64, measurement.sample_count.max(1), false),
        },
        Some(v) => (v, measurement.sample_count, false),
    };
    let _ = skipped;

    // Below min samples: do not advance counters (and never open/resolve).
    if sample_count < config.minimum_sample_count {
        let mut state = prev.clone();
        state.last_value = Some(effective_value);
        state.last_sample_count = sample_count;
        return EvaluationOutcome {
            state,
            transition: AlertTransition::None,
            is_breach: false,
            effective_value: Some(effective_value),
        };
    }

    let is_breach = value_breaches(config, effective_value);
    let mut state = prev.clone();
    state.last_value = Some(effective_value);
    state.last_sample_count = sample_count;

    if is_breach {
        state.consecutive_breaches = state.consecutive_breaches.saturating_add(1);
        state.consecutive_healthy = 0;
    } else {
        state.consecutive_healthy = state.consecutive_healthy.saturating_add(1);
        state.consecutive_breaches = 0;
    }

    let transition = if is_breach {
        if !state.incident_open && state.consecutive_breaches >= breaches_required {
            state.incident_open = true;
            state.last_notified_at = Some(now_unix_secs);
            AlertTransition::OpenIncident
        } else if state.incident_open {
            // Renotify only while still open and interval elapsed.
            let interval_secs = i64::from(config.renotify_interval_minutes) * 60;
            let due = match state.last_notified_at {
                None => true,
                Some(t) => now_unix_secs.saturating_sub(t) >= interval_secs,
            };
            if due && interval_secs > 0 {
                state.last_notified_at = Some(now_unix_secs);
                AlertTransition::Renotify
            } else if due && interval_secs == 0 {
                // Zero interval = renotify every breach tick while open.
                state.last_notified_at = Some(now_unix_secs);
                AlertTransition::Renotify
            } else {
                AlertTransition::None
            }
        } else {
            AlertTransition::None
        }
    } else if state.incident_open && state.consecutive_healthy >= healthy_required {
        state.incident_open = false;
        state.last_notified_at = None;
        AlertTransition::ResolveIncident
    } else {
        AlertTransition::None
    };

    EvaluationOutcome {
        state,
        transition,
        is_breach,
        effective_value: Some(effective_value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(threshold: f64) -> RuleEvalConfig {
        RuleEvalConfig {
            threshold,
            consecutive_breaches_required: 2,
            consecutive_healthy_required: 2,
            minimum_sample_count: 1,
            renotify_interval_minutes: 30,
            ..RuleEvalConfig::default()
        }
    }

    fn m(value: f64, n: u64) -> AlertMeasurement {
        AlertMeasurement {
            value: Some(value),
            sample_count: n,
        }
    }

    #[test]
    fn breach_hysteresis_requires_consecutive() {
        let config = cfg(0.2);
        let mut state = RuleEvalState::default();
        let o1 = evaluate_rule(&config, &state, m(0.5, 10), 1_000);
        assert_eq!(o1.transition, AlertTransition::None);
        assert!(!o1.state.incident_open);
        assert_eq!(o1.state.consecutive_breaches, 1);
        state = o1.state;

        let o2 = evaluate_rule(&config, &state, m(0.6, 10), 1_060);
        assert_eq!(o2.transition, AlertTransition::OpenIncident);
        assert!(o2.state.incident_open);
        assert_eq!(o2.state.consecutive_breaches, 2);
    }

    #[test]
    fn healthy_hysteresis_resolves_incident() {
        let config = cfg(0.2);
        let mut state = RuleEvalState {
            consecutive_breaches: 2,
            incident_open: true,
            last_notified_at: Some(1_000),
            ..RuleEvalState::default()
        };
        let o1 = evaluate_rule(&config, &state, m(0.01, 10), 2_000);
        assert_eq!(o1.transition, AlertTransition::None);
        assert!(o1.state.incident_open);
        state = o1.state;

        let o2 = evaluate_rule(&config, &state, m(0.0, 10), 2_060);
        assert_eq!(o2.transition, AlertTransition::ResolveIncident);
        assert!(!o2.state.incident_open);
        assert_eq!(o2.state.last_notified_at, None);
    }

    #[test]
    fn flapping_never_double_opens_while_open() {
        let config = cfg(0.2);
        let mut state = RuleEvalState::default();
        // Open.
        state = evaluate_rule(&config, &state, m(0.5, 5), 100).state;
        let open = evaluate_rule(&config, &state, m(0.5, 5), 160);
        assert_eq!(open.transition, AlertTransition::OpenIncident);
        state = open.state;
        // Single healthy tick resets breach counter but keeps incident open.
        let heal = evaluate_rule(&config, &state, m(0.0, 5), 220);
        assert_eq!(heal.transition, AlertTransition::None);
        assert!(heal.state.incident_open);
        state = heal.state;
        // Breach again while still open — must not OpenIncident again.
        let rebreach = evaluate_rule(&config, &state, m(0.9, 5), 280);
        assert_ne!(rebreach.transition, AlertTransition::OpenIncident);
        assert!(rebreach.state.incident_open);
        // Second consecutive breach while open still no double-open.
        let rebreach2 = evaluate_rule(&config, &rebreach.state, m(0.9, 5), 340);
        assert_ne!(rebreach2.transition, AlertTransition::OpenIncident);
        assert!(rebreach2.state.incident_open);
    }

    #[test]
    fn min_sample_count_blocks_transitions() {
        let config = RuleEvalConfig {
            minimum_sample_count: 10,
            consecutive_breaches_required: 1,
            ..cfg(0.1)
        };
        let state = RuleEvalState::default();
        let o = evaluate_rule(&config, &state, m(0.9, 3), 1_000);
        assert_eq!(o.transition, AlertTransition::None);
        assert!(!o.state.incident_open);
        assert_eq!(o.state.consecutive_breaches, 0);
    }

    #[test]
    fn no_data_skip_leaves_state() {
        let config = RuleEvalConfig {
            no_data_behavior: NoDataBehavior::Skip,
            consecutive_breaches_required: 1,
            ..cfg(0.1)
        };
        let prev = RuleEvalState {
            consecutive_breaches: 1,
            ..RuleEvalState::default()
        };
        let o = evaluate_rule(
            &config,
            &prev,
            AlertMeasurement {
                value: None,
                sample_count: 0,
            },
            1_000,
        );
        assert_eq!(o.transition, AlertTransition::None);
        assert_eq!(o.state.consecutive_breaches, 1);
        assert_eq!(o.effective_value, None);
    }

    #[test]
    fn no_data_zero_can_breach() {
        let config = RuleEvalConfig {
            no_data_behavior: NoDataBehavior::Zero,
            comparator: AlertComparator::Lte,
            threshold: 0.0,
            consecutive_breaches_required: 1,
            minimum_sample_count: 1,
            ..RuleEvalConfig::default()
        };
        let o = evaluate_rule(
            &config,
            &RuleEvalState::default(),
            AlertMeasurement {
                value: None,
                sample_count: 0,
            },
            1_000,
        );
        assert!(o.is_breach);
        assert_eq!(o.transition, AlertTransition::OpenIncident);
        assert_eq!(o.effective_value, Some(0.0));
    }

    #[test]
    fn between_and_not_between() {
        let between = RuleEvalConfig {
            comparator: AlertComparator::Between,
            threshold: 10.0,
            threshold_upper: Some(20.0),
            consecutive_breaches_required: 1,
            ..RuleEvalConfig::default()
        };
        assert!(value_breaches(&between, 15.0));
        assert!(!value_breaches(&between, 9.0));
        assert!(!value_breaches(&between, 21.0));

        let not_between = RuleEvalConfig {
            comparator: AlertComparator::NotBetween,
            threshold: 10.0,
            threshold_upper: Some(20.0),
            consecutive_breaches_required: 1,
            ..RuleEvalConfig::default()
        };
        assert!(value_breaches(&not_between, 9.0));
        assert!(!value_breaches(&not_between, 15.0));
    }

    #[test]
    fn renotify_interval_gate() {
        let config = RuleEvalConfig {
            renotify_interval_minutes: 30,
            consecutive_breaches_required: 1,
            ..cfg(0.1)
        };
        let mut state = RuleEvalState::default();
        let open = evaluate_rule(&config, &state, m(0.5, 5), 1_000);
        assert_eq!(open.transition, AlertTransition::OpenIncident);
        state = open.state;

        // 10 minutes later — not due.
        let mid = evaluate_rule(&config, &state, m(0.5, 5), 1_000 + 10 * 60);
        assert_eq!(mid.transition, AlertTransition::None);
        state = mid.state;

        // 30 minutes after last notify — renotify.
        let due = evaluate_rule(&config, &state, m(0.5, 5), 1_000 + 30 * 60);
        assert_eq!(due.transition, AlertTransition::Renotify);
        assert_eq!(due.state.last_notified_at, Some(1_000 + 30 * 60));
    }

    #[test]
    fn comparators_gt_gte_lt_lte() {
        let base = RuleEvalConfig {
            threshold: 5.0,
            consecutive_breaches_required: 1,
            ..RuleEvalConfig::default()
        };
        assert!(value_breaches(
            &RuleEvalConfig {
                comparator: AlertComparator::Gt,
                ..base.clone()
            },
            5.1
        ));
        assert!(!value_breaches(
            &RuleEvalConfig {
                comparator: AlertComparator::Gt,
                ..base.clone()
            },
            5.0
        ));
        assert!(value_breaches(
            &RuleEvalConfig {
                comparator: AlertComparator::Gte,
                ..base.clone()
            },
            5.0
        ));
        assert!(value_breaches(
            &RuleEvalConfig {
                comparator: AlertComparator::Lt,
                ..base.clone()
            },
            4.9
        ));
        assert!(value_breaches(
            &RuleEvalConfig {
                comparator: AlertComparator::Lte,
                ..base
            },
            5.0
        ));
    }
}
