use core::num;
use std::error::Error;

pub struct SoundManager {
    frequency: f64,
    amplitude: f64,
    intensity: f64,
    duration: f64,
    brightness: f64,
    roughness: f64,
    sharpness: f64,
    attack: f64,
    decay: f64,
    sample_rate: f64
}


#[derive(Debug)]
enum FormatError {
    ValueTooHigh,
    ValueTooLow,
    ValueError
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::ValueError => write!(f, "An error has occured"),
            FormatError::ValueTooHigh => write!(f, "Value submitted it too high"),
            FormatError::ValueTooLow => write!(f, "Value submitted it too low"),

        }
    }
}

impl std::error::Error for FormatError {}

fn in_range<T>(target: T, min: T, max: T) -> Result<T, Box<dyn Error>> where T: PartialOrd {
    if target <= max && target >= min {
        Ok(target)
    } else {
        if target < max {
            Err(Box::new(FormatError::ValueTooLow))
        } else if target > min {
            Err(Box::new(FormatError::ValueTooLow))
            
        } else {
            Err(Box::new(FormatError::ValueError)) // theoratically, should never reach here
        }
    }
}

impl SoundManager {
    pub fn set(
        &self,
        frequency: Option<f64>,   // Provide a frequency in hz (hertz)
        amplitude: Option<f64>,   // Scale of volume, where 1.0 = max volume
        intensity: Option<f64>,   // Energy scale factor, 0.0 to 1.0
        duration: Option<f64>,    // Time to play audio in seconds
        brightness: Option<f64>,  // Controls harmonics, 0.0 to 1.0
        roughness: Option<f64>,   // Controls "gritiness", 0.0 to 1.0
        sharpness: Option<f64>,   // Controls "metalitic" noise (like when high pitches have that annoying 
                                  // screeching sound), 0.0 to 1.0

        attack: Option<f64>,      // Time for fade in
        decay: Option<f64>,       // Time for fade out
        sample_rate: Option<u32>  // Sample rate in number of samples (bits to represent each sound wave) taken per second
    ) -> Result<(), Box<dyn Error>> {
        let frequency = frequency.unwrap_or(500.0);
        let amplitude = in_range(amplitude.unwrap_or(0.0), 0.0, 1.0)?;
        let duration = duration.unwrap_or(3.0);
        let intensity = in_range(intensity.unwrap_or(0.0), 0.0, 1.0)?;
        let brightness = in_range(brightness.unwrap_or(0.0), 0.0, 1.0)?;
        let roughness = in_range(roughness.unwrap_or(0.0), 0.0, 1.0)?;
        let sharpness = in_range(sharpness.unwrap_or(0.0), 0.0, 1.0)?;
        let attack = attack.unwrap_or(0.0);
        let decay = decay.unwrap_or(0.0);
        let sample_rate = sample_rate.unwrap_or(44100);

        /*
            Sample Number = Sampling rate * Duration 
        */
        let samples = (sample_rate as f64 * duration).round() as usize;
        let mut signal = vec![0.0; samples];
        let num_harmonics = (1.0 + (brightness * 12.0)).round() as i64; // unwrap should be safe here
        for i in 1..=samples {
            let t = i as f64 / sample_rate as f64;
            let mut sample_val = 0.0;
            for harmony in 1..=num_harmonics {
                let h = harmony as f64;
                let pos = h/num_harmonics as f64; /* 
                    Get the position relative to the highest harmony
                */
                let adjusted_pos = sharpness * pos; // adjust the effect of the position with consideration of sharpness
                let balanced_pos = 1.0 + adjusted_pos; // create a minimum of 1.0 base weight
                let total_weight =  (1.0/h) * balanced_pos; 

                /* 
                    Think of it like this,
                        i want a sharpness of 0.3 or 30% or 130%,
                        adjusted_pos is just going to give us the 30%,
                        if we set sharpness to 0.0 (meaning, no effect to base harmony), the harmony would be nulled,
                        instead with balanced_pos, that "30%" from adjusted_pos is
                        turned into 100% + 30% -> 30% (meaning if we gave 0.0, 100% + 0% -> 100% (no effect))
                        total_weight ensures higher harmonies get less weight
                */
                let f = frequency * h;
                let this_harmonic_sample_value = total_weight  * (2.0 * std::f64::consts::PI * f * t).sin();
                /* 
                    (2.0 * std::f64::consts::PI * f * t).sin()
                    is basically:
                        sin(2 * pi * frequency * time) 
                    the formula is used to find how far up the graph is on the y axis at a
                    specific time

                    f is frequency * h, because we need to account for the harmonic
                */
                sample_val += this_harmonic_sample_value;

            }
            signal[i] = sample_val // store the val in the array (so we can plot on graph then find the wave)
        }

        if roughness > 0.0 {
            let beat_rate = 15.0 + (roughness * 35.0); // creates a range from 15 to 35
            for (index, sample) in signal.iter_mut().enumerate() {
                let time = index as f64 / sample_rate as f64;
                let cycles = (2.0 * std::f64::consts::PI * beat_rate * time).sin();
                /* 
                    (2.0 * std::f64::consts::PI * beat_rate * t).sin()
                    is basically:
                        sin(2 * pi * frequency * time) 
                    the formula is used to find how far up the graph is on the y axis at a
                    specific time
                    _NOTE: range right now the range (-1.0, 1.0) we want positive numbers
                */
                let positive_cycles = cycles + 1.0;
                /*
                    + 1.0 increases the range to positive values:
                        Lower bound: -1.0 + 1.0 = 0.0
                        Upper bound: +1.0 + 1.0 = 2.0
                    new range (0.0, 2.0)
                    You might be thinking "cant i use abs()", here is why you cant use it:
                        Lower bound: abs(-1.0) -> 1.0
                        Upper bound: abs(+1.0) -> 1.0
                    now the new "range" is 1.0 to 1.0
                    but the thing is, its actually 0.0 to 1.0 because:
                        Middle bound: abs(0.0) -> 0.0
                    0.0 to 1.0 reduces the range (since the old range is much bigger, -1.0 to 1.0,
                    we basically got "rid" half of the range)
                */
                let max_volume_reduction = roughness * 0.4;
                let modulator = 1.0 - (max_volume_reduction * positive_cycles);
                *sample *= modulator;
            }
        }

        Ok(())
    }
}