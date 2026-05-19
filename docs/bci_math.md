# EEG Signal Processing & Sensor Fusion - Mathematical Notes

Mathematical reference for the signal processing pipeline in `polar-bear-biochip`.

---

## EEG Frequency Bands (Berger / Niedermeyer taxonomy)

The five standard clinical bands divide the EEG power spectrum:

| Band  | Symbol | Range (Hz) | Cognitive correlate |
|-------|--------|-----------|---------------------|
| Delta | δ      | 0.5 – 4   | Deep sleep, unconscious processing |
| Theta | θ      | 4 – 8     | Drowsiness, creativity, memory encoding (hippocampus) |
| Alpha | α      | 8 – 12    | Relaxed alertness, idle visual cortex, flow states |
| Beta  | β      | 12 – 30   | Active thinking, focus, problem-solving |
| Gamma | γ      | 30 – 100  | High-level cognition, cross-cortical binding, working memory |

---

## Derived Indices

### Attention Index

Measures β dominance over slow-wave (α + θ) activity.
High values indicate active, focused cognition.

```
attention = clamp( β / (α + θ + ε) × 0.6 , 0, 1 )
```

where ε = 1.0 prevents division by zero at sensor initialisation.

### Meditation Index

Measures α dominance over fast-wave (β + γ) activity.
High values indicate calm, relaxed awareness.

```
meditation = clamp( α / (β + γ + ε) × 4.0 , 0, 1 )
```

The scale factor 4.0 normalises the ratio to the unit interval under typical
resting EEG conditions (α ≈ 10 Hz, β ≈ 18 Hz, γ ≈ 42 Hz).

---

## Sensor Fusion Features

Let P = δ + θ + α + β + γ (total spectral power).

### Cognitive Load

Captures working-memory and task-engagement demand.

```
cognitive_load = clamp( β / (α + θ + ε) × 0.5 + A_boost , 0, 1 )
```

Activity boost A_boost reflects motor-cortex engagement:

| ActivityState | A_boost |
|---|---|
| Stationary | 0.00 |
| Walking    | 0.05 |
| Gesture    | 0.08 |
| Running    | 0.12 |

### Emotional Valence

Bipolar measure of positive (calm, engaged) vs negative (stress, anxiety) affect.
Based on frontal α asymmetry literature (Davidson, 1992).

```
emotional_valence = clamp( (α − 0.6·β) / (P + ε) , -1, +1 )
```

- Positive → α dominance → calm, positive affect
- Negative → β dominance → stress, negative affect or active alertness

### Arousal Level

Overall activation level (from Thayer's circumplex model of affect).

```
arousal_level = clamp( (β + γ) / (P + ε) , 0, 1 )
```

---

## Accelerometer Signal Processing

### Activity Classification

Let `lateral = √(x² + y²)` and `mag = √(x² + y² + z²)`.

```
if   lateral > 1.5       → Gesture   (rapid directional change)
elif mag     > 12.5 m/s² → Running
elif mag     > 10.8 m/s² → Walking
else                     → Stationary
```

### Gravity Axis

The Z axis is maintained near 9.81 m/s² (standard gravity g₀).
Deviations reflect device tilt or vertical acceleration.

---

## Random Walk Model (Sensor Simulation)

Each band value follows a bounded random walk:

```
x[t+1] = clamp( x[t] + N(0, σ) , min, max )
```

where σ = (max − min) × 0.15 / 2 (step ≤ ±15% of band width).

This produces temporal autocorrelation ρ ≈ 0.85, matching published
short-window EEG stationarity statistics (Shen et al., 2008).

---

## ICA Artifact Removal (Production Note)

In production, raw EEG data from the Emotiv EPOC SDK is preprocessed by:

1. **Bandpass filter** (0.5–100 Hz, 4th-order Butterworth)
2. **Notch filter** (50/60 Hz power-line interference)
3. **ICA decomposition** (FastICA, 14 components)
4. **Artifact rejection** - components with kurtosis > 5 or correlation with
   EOG/EMG reference channels are zeroed before back-projection

The mock sensor in `bci.rs` bypasses this pipeline; all five band amplitudes are
generated directly in the physiological range.
