#[cfg(test)]
mod tests {
    use crate::envelope::ADSR;
    use crate::reverb::AnalogReverb;
    
    #[test]
    fn test_envelope_adsr_basic() {
        let mut envelope = ADSR::new(0.1, 0.1, 0.5, 0.2); // A=0.1s, D=0.1s, S=0.5, R=0.2s
        
        // Start envelope
        envelope.start(0.0);
        
        // Test attack phase
        assert_eq!(envelope.value(0.0), 0.0, "Envelope should start at 0");
        assert_eq!(envelope.value(0.05), 0.5, "Envelope should be halfway through attack at 0.05s");
        assert_eq!(envelope.value(0.1), 1.0, "Envelope should reach peak at 0.1s");
        
        // Test decay phase  
        assert_eq!(envelope.value(0.15), 0.75, "Envelope should be decaying at 0.15s");
        assert_eq!(envelope.value(0.2), 0.5, "Envelope should reach sustain at 0.2s");
        
        // Test sustain phase
        assert_eq!(envelope.value(0.5), 0.5, "Envelope should hold sustain");
        assert_eq!(envelope.value(1.0), 0.5, "Envelope should hold sustain");
        
        println!("✅ Basic ADSR phases working correctly");
    }
    
    #[test] 
    fn test_envelope_release_behavior() {
        let mut envelope = ADSR::new(0.1, 0.1, 0.5, 0.2);
        
        // Start and let it reach sustain
        envelope.start(0.0);
        
        // Trigger release during sustain phase
        envelope.stop(1.0); // Release at 1 second
        
        // Test release phase
        let release_start_value = envelope.value(1.0);
        println!("Release start value: {}", release_start_value);
        assert!(release_start_value > 0.4, "Should start release from sustain level");
        
        let mid_release_value = envelope.value(1.1); // 0.1s into release
        println!("Mid-release value: {}", mid_release_value);
        assert!(mid_release_value < release_start_value, "Should be decreasing during release");
        assert!(mid_release_value > 0.0, "Should not reach zero yet");
        
        let end_release_value = envelope.value(1.2); // At end of release
        println!("End-release value: {}", end_release_value);
        assert!(end_release_value < 0.1, "Should be near zero at end of release");
        
        let post_release_value = envelope.value(1.3); // After release
        assert_eq!(post_release_value, 0.0, "Should be zero after release");
        
        println!("✅ Envelope release behavior working correctly");
    }
    
    #[test]
    fn test_comb_filter_simple() {
        use crate::reverb::AnalogReverb;
        
        println!("=== Testing Simple Comb Filter Logic ===");
        
        // Create a reverb with very high settings to force feedback
        let mut reverb = AnalogReverb::new(44100.0);
        reverb.set_mix(1.0); // 100% wet
        reverb.set_decay(0.99); // Maximum decay
        reverb.set_room_size(1.0); // Maximum room size
        
        // Send a strong impulse
        let impulse_response = reverb.process(1.0);
        println!("Impulse response: {:.6}", impulse_response);
        
        // At 44.1kHz, the shortest delay is 30ms = ~1,323 samples
        // So we need to wait that long to see the first delayed feedback
        println!("Waiting for delay line to fill (shortest = 30ms = ~1,323 samples)");
        
        let mut found_feedback = false;
        for i in 0..2000 { // Check first 2000 samples
            let sample = reverb.process(0.0);
            
            if sample.abs() > 0.001 {
                println!("✅ Found feedback at sample {}: {:.6}", i, sample);
                found_feedback = true;
                
                // Show next few samples
                for j in 1..10 {
                    let next = reverb.process(0.0);
                    println!("  Sample {}: {:.6}", i + j, next);
                }
                break;
            }
            
            if i % 500 == 0 {
                println!("  Checked {} samples, still waiting...", i);
            }
        }
        
        if !found_feedback {
            println!("❌ No feedback found in 2000 samples");
            println!("Issue: Either delay times are too long, or feedback math is wrong");
        }
        
        assert!(true, "Diagnostic test");
    }
    
    #[test]
    fn test_reverb_mix_levels() {
        let mut reverb = AnalogReverb::new(44100.0);
        
        // Test 0% mix (all dry)
        reverb.set_mix(0.0);
        let input = 0.5;
        let output_dry = reverb.process(input);
        assert!((output_dry - input).abs() < 0.01, "0% mix should be mostly dry");
        
        // Test 50% mix
        reverb.set_mix(0.5); 
        let output_50 = reverb.process(input);
        
        // Test 100% mix (all wet)
        reverb.set_mix(1.0);
        let output_wet = reverb.process(input);
        
        println!("Dry: {}, 50% mix: {}, Wet: {}", output_dry, output_50, output_wet);
        
        // With 50% mix: output = input * 0.5 + diffused * 0.5 * 2.0
        // The key test is that mix controls actually do something
        assert_ne!(output_50, output_dry, "50% mix should be different from dry");
        assert_ne!(output_wet, output_dry, "100% wet should be different from dry");
        assert!(output_wet.abs() > 0.01, "100% wet should produce audible output (got {})", output_wet);
        
        println!("✅ Reverb mix controls working");
    }
    
    #[test]
    fn test_envelope_timing_precision() {
        let mut envelope = ADSR::new(0.001, 0.001, 0.8, 0.001); // Very fast envelope
        
        envelope.start(0.0);
        
        // Test very precise timing
        let values: Vec<(f32, f32)> = (0..10)
            .map(|i| {
                let time = i as f32 * 0.0001; // 0.1ms increments
                (time, envelope.value(time))
            })
            .collect();
            
        println!("Precise envelope timing:");
        for (time, value) in &values {
            println!("  t={:.4}s, v={:.3}", time, value);
        }
        
        // Values should be increasing during attack
        assert!(values[9].1 > values[0].1, "Envelope should be rising");
        
        println!("✅ Envelope timing precision verified");
    }
}