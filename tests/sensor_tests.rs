//! Integration tests - sensor layer

use polar_bear_biochip::{
    sensors::{
        accelerometer::AccelerometerSensor,
        bci::BciSensor,
        fusion::SensorFusion,
    },
    types::ActivityState,
};

// ── BCI sensor ────────────────────────────────────────────────────────────────

#[test]
fn bci_produces_valid_bands_over_many_samples() {
    let mut sensor = BciSensor::new();
    for _ in 0..200 {
        let r = sensor.sample();
        assert!((0.5..=4.0).contains(&r.delta_hz), "delta: {}", r.delta_hz);
        assert!((4.0..=8.0).contains(&r.theta_hz), "theta: {}", r.theta_hz);
        assert!((8.0..=12.0).contains(&r.alpha_hz), "alpha: {}", r.alpha_hz);
        assert!((12.0..=30.0).contains(&r.beta_hz), "beta: {}", r.beta_hz);
        assert!((30.0..=70.0).contains(&r.gamma_hz), "gamma: {}", r.gamma_hz);
    }
}

#[test]
fn bci_attention_index_within_unit_interval() {
    let mut sensor = BciSensor::new();
    for _ in 0..100 {
        let r = sensor.sample();
        assert!(
            (0.0..=1.0).contains(&r.attention_index),
            "attention out of [0,1]: {}",
            r.attention_index
        );
    }
}

#[test]
fn bci_meditation_index_within_unit_interval() {
    let mut sensor = BciSensor::new();
    for _ in 0..100 {
        let r = sensor.sample();
        assert!(
            (0.0..=1.0).contains(&r.meditation_index),
            "meditation out of [0,1]: {}",
            r.meditation_index
        );
    }
}

// ── Accelerometer sensor ──────────────────────────────────────────────────────

#[test]
fn accelerometer_gravity_axis_within_bounds() {
    let mut sensor = AccelerometerSensor::new();
    for _ in 0..200 {
        let r = sensor.sample();
        assert!((8.5..=11.0).contains(&r.z), "z out of gravity range: {}", r.z);
    }
}

#[test]
fn accelerometer_magnitude_always_positive() {
    let mut sensor = AccelerometerSensor::new();
    for _ in 0..100 {
        let r = sensor.sample();
        assert!(r.magnitude > 0.0, "magnitude must be positive: {}", r.magnitude);
    }
}

#[test]
fn accelerometer_activity_states_are_valid_variants() {
    let mut sensor = AccelerometerSensor::new();
    for _ in 0..100 {
        let r = sensor.sample();
        assert!(matches!(
            r.activity_state,
            ActivityState::Stationary
                | ActivityState::Walking
                | ActivityState::Running
                | ActivityState::Gesture
        ));
    }
}

// ── Sensor fusion ─────────────────────────────────────────────────────────────

#[test]
fn fusion_cognitive_load_within_unit_interval() {
    let mut fusion = SensorFusion::new();
    for id in 1..=100_u64 {
        let r = fusion.sample(id);
        assert!(
            (0.0..=1.0).contains(&r.cognitive_load),
            "cognitive_load out of [0,1]: {}",
            r.cognitive_load
        );
    }
}

#[test]
fn fusion_emotional_valence_within_bipolar_interval() {
    let mut fusion = SensorFusion::new();
    for id in 1..=100_u64 {
        let r = fusion.sample(id);
        assert!(
            (-1.0..=1.0).contains(&r.emotional_valence),
            "emotional_valence out of [-1,1]: {}",
            r.emotional_valence
        );
    }
}

#[test]
fn fusion_arousal_within_unit_interval() {
    let mut fusion = SensorFusion::new();
    for id in 1..=100_u64 {
        let r = fusion.sample(id);
        assert!(
            (0.0..=1.0).contains(&r.arousal_level),
            "arousal out of [0,1]: {}",
            r.arousal_level
        );
    }
}

#[test]
fn fusion_sequence_id_is_preserved_exactly() {
    let mut fusion = SensorFusion::new();
    for id in [1_u64, 7, 100, 99_999, u64::MAX / 4] {
        assert_eq!(fusion.sample(id).sequence_id, id);
    }
}

#[test]
fn fusion_timestamps_are_monotonically_non_decreasing() {
    let mut fusion = SensorFusion::new();
    let mut prev   = fusion.sample(1).timestamp;
    for id in 2..=20_u64 {
        let curr = fusion.sample(id).timestamp;
        assert!(curr >= prev, "timestamp went backwards at id={id}");
        prev = curr;
    }
}
