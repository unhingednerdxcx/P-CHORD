use hound::{WavSpec, WavWriter};
use std::f64::consts::PI;

fn generate_custom_sound(
    frequency: f64,      // Pitch in Hz (e.g., 440 = A4)
    amplitude: f64,      // Peak volume scaling (0.0 to 1.0)
    intensity: f64,      // Energy scaling factor (0.0 to 1.0)
    duration: f64,       // Total duration in seconds
    brightness: f64,     // Controls higher harmonic content (0.0 to 1.0)
    roughness: f64,      // Controls rapid amplitude modulation/beating (0.0 to 1.0)
    sharpness: f64,      // Emphasizes high-frequency spectral weight (0.0 to 1.0)
    attack_time: f64,    // Time to reach peak volume in seconds
    decay_time: f64,     // Time to drop from peak to sustain level in seconds
    sample_rate: u32,    // Standard audio sample rate
) -> (u32, Vec<i16>) {
    let num_samples = (sample_rate as f64 * duration) as usize;
    let mut signal = vec![0.0; num_samples];

    // 1. Brightness & Sharpness: Additive Synthesis (Harmonics)
    let num_harmonics = 1 + (brightness * 12.0) as usize;

    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let mut sample_val = 0.0;

        for h in 1..=num_harmonics {
            let h_f64 = h as f64;
            // Sharpness applies extra weight to higher frequency harmonics
            let weight = (1.0 / h_f64) * (1.0 + sharpness * (h_f64 / num_harmonics as f64));
            sample_val += weight * (2.0 * PI * frequency * h_f64 * t).sin();
        }
        signal[i] = sample_val;
    }

    // Find maximum absolute value for normalization
    let mut max_val = 0.0;
    for &s in &signal {
        if s.abs() > max_val {
            max_val = s.abs();
        }
    }

    // Normalize base waveform
    if max_val > 0.0 {
        for s in &mut signal {
            *s /= max_val;
        }
    }

    // 2. Roughness: Apply rapid amplitude modulation (tremolo / beating)
    if roughness > 0.0 {
        let mod_freq = 15.0 + (roughness * 35.0); // Modulation between 15 Hz and 50 Hz
        for (i, s) in signal.iter_mut().enumerate() {
            let t = i as f64 / sample_rate as f64;
            let modulator = 1.0 - (roughness * 0.4 * (1.0 + (2.0 * PI * mod_freq * t).sin()));
            *s *= modulator;
        }
    }

    // 3. Attack & Decay (ADSR Envelope)
    let attack_samples = (attack_time * sample_rate as f64) as usize;
    let decay_samples = (decay_time * sample_rate as f64) as usize;
    let sustain_level = 0.7;
    let release_samples = (0.1 * sample_rate as f64) as usize;

    let mut envelope = vec![1.0; num_samples];

    // Attack ramp up
    for i in 0..attack_samples.min(num_samples) {
        envelope[i] = i as f64 / attack_samples as f64;
    }

    // Decay ramp down to sustain level
    let end_decay = (attack_samples + decay_samples).min(num_samples);
    for i in attack_samples..end_decay {
        let progress = (i - attack_samples) as f64 / decay_samples as f64;
        envelope[i] = 1.0 - progress * (1.0 - sustain_level);
    }
    for i in end_decay..num_samples {
        envelope[i] = sustain_level;
    }

    // Release ramp down (final 10% of duration)
    if release_samples < num_samples {
        let start_release = num_samples - release_samples;
        for i in start_release..num_samples {
            let progress = (i - start_release) as f64 / release_samples as f64;
            envelope[i] *= 1.0 - progress;
        }
    }

    // Apply envelope, amplitude, and intensity, then convert to 16-bit PCM
    let total_amplitude = (amplitude * intensity).clamp(0.0, 1.0);
    let mut audio_data = vec![0i16; num_samples];

    for i in 0..num_samples {
        let final_sample = signal[i] * envelope[i] * total_amplitude;
        // Scale to i16 range (-32768 to 32767)
        audio_data[i] = (final_sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
    }

    (sample_rate, audio_data)
}

fn main() {
    let (sr, data) = generate_custom_sound(
        300.0, // frequency
        0.9,   // amplitude
        0.8,   // intensity
        3.0,   // duration
        0.7,   // brightness
        0.3,   // roughness
        0.6,   // sharpness
        0.1,   // attack_time
        0.3,   // decay_time
        44100, // sample_rate
    );

    // Save to a playable WAV file using Hound
    let spec = WavSpec {
        channels: 1,
        sample_rate: sr,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create("custom_generated_sound.wav", spec).unwrap();
    for &sample in &data {
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();

    println!("Sound generated and saved as 'custom_generated_sound.wav' in Rust!");
}