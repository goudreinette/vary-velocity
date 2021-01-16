#[macro_use]
extern crate vst;
extern crate time;
extern crate rand;
extern crate rand_distr;

use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Info, Plugin, PluginParameters};
use vst::util::AtomicFloat;
use vst::api;
use vst::buffer::{ SendEventBuffer};
use vst::event::{Event, MidiEvent};
use vst::plugin::{CanDo, HostCallback,};
use std::sync::Arc;
use rand_distr::{Normal, Distribution};


/**
 * Parameters
 */ 
struct VaryVelocityParameters {
    variance: AtomicFloat,
}


impl Default for VaryVelocityParameters {
    fn default() -> VaryVelocityParameters {
        VaryVelocityParameters {
            variance: AtomicFloat::new(0.0),
        }
    }
}


static MAX_VARIANCE: f32 = 5.;


/**
 * Plugin
 */ 

#[derive(Default)]
struct VaryVelocity {
    host: HostCallback,
    sample_rate: f32,
    immediate_events: Vec<MidiEvent>,
    send_buffer: SendEventBuffer,
    params: Arc<VaryVelocityParameters>,
}


impl VaryVelocity {
    fn add_event(&mut self, e: MidiEvent) {
        let velocity = e.data[2];
        let variance = self.params.variance.get() * 3.;

        let normal = Normal::new(velocity as f32, variance).unwrap();
        let v = normal.sample(&mut rand::thread_rng()).max(0.).min(127.) as f32;

        self.immediate_events.push(MidiEvent {
            data: [e.data[0], e.data[1], v as u8],
            ..e
        });
    }
    
    fn send_midi(&mut self) {
        // Immediate
        self.send_buffer.send_events(&self.immediate_events, &mut self.host);
        self.immediate_events.clear();
    }
}

impl Plugin for VaryVelocity {
    fn new(host: HostCallback) -> Self {
        let mut p = VaryVelocity::default();
        p.host = host;
        p.params = Arc::new(VaryVelocityParameters::default());
        p
    }

    fn get_info(&self) -> Info {
        Info {
            name: "VaryVelocity".to_string(),
            vendor: "Rein van der Woerd".to_string(),
            unique_id: 243723072,
            version: 1,
            inputs: 2,
            outputs: 2,
            // This `parameters` bit is important; without it, none of our
            // parameters will be shown!
            parameters: 1,
            category: Category::Effect,
            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
    }

    fn process_events(&mut self, events: &api::Events) {
        for e in events.events() {
            #[allow(clippy::single_match)]
            match e {
                Event::Midi(e) => self.add_event(e),
                _ => (),
            }
        }
    }


    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        for (input, output) in buffer.zip() {
            for (in_sample, out_sample) in input.iter().zip(output) {
                *out_sample = *in_sample;
            }
        }
        self.send_midi();
    }

    fn can_do(&self, can_do: CanDo) -> vst::api::Supported {
        use vst::api::Supported::*;
        use vst::plugin::CanDo::*;

        match can_do {
            SendEvents | SendMidiEvent | ReceiveEvents | ReceiveMidiEvent => Yes,
            _ => No,
        }
    }


    // Return the parameter object. This method can be omitted if the
    // plugin has no parameters.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl PluginParameters for VaryVelocityParameters {
    // the `get_parameter` function reads the value of a parameter.
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.variance.get() * MAX_VARIANCE,
            _ => 0.0,
        }
    }

    // the `set_parameter` function sets the value of a parameter.
    fn set_parameter(&self, index: i32, val: f32) {
        #[allow(clippy::single_match)]
        match index {
            0 => self.variance.set(val.max(0.0000000001) * MAX_VARIANCE),
            _ => (),
        }
    }

    // This is what will display underneath our control.  We can
    // format it into a string that makes the most since.
    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 =>  format!("{:.1}", self.variance .get()),
            _ => "".to_string(),
        }
    }

    // This shows the control's name.
    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Velocity variance",
            _ => "",
        }
        .to_string()
    }
}

// This part is important!  Without it, our plugin won't work.
plugin_main!(VaryVelocity);