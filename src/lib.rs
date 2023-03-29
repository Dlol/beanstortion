use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::{sync::Arc, default, fmt::Display};

mod editor;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct YasYas {
    params: Arc<YasYasParams>,
    CLIPPING_FAC: f32
    // luts: [[f32; ]]
}

#[derive(Params)]
struct YasYasParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "clip"]
    pub clip: FloatParam,

    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "distType"]
    pub dist_type: EnumParam<DistTypes>,

    #[id = "mix"]
    pub mix: FloatParam,

    #[id = "autogain"]
    pub autogain: BoolParam
}

impl Default for YasYas {
    fn default() -> Self {
        Self {
            params: Arc::new(YasYasParams::default()),
            CLIPPING_FAC: util::db_to_gain(0.0)
        }
    }
}

impl Default for YasYasParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            clip: FloatParam::new(
                "clip",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-10.0),
                    max: util::db_to_gain(40.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-10.0, 40.0),
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-60.0),
                    max: util::db_to_gain(5.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-60.0, 5.0),
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            dist_type: EnumParam::new(
                "Distortion Type",
                DistTypes::HardClip
            ),
            mix: FloatParam::new(
                "Mix",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ),
            autogain: BoolParam::new("Auto Gain", false)
        }
    }
}

impl Plugin for YasYas {
    const NAME: &'static str = "beanstortion";
    const VENDOR: &'static str = "dlol";
    const URL: &'static str = "https://youtu.be/dQw4w9WgXcQ";
    const EMAIL: &'static str = "your@email.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const DEFAULT_INPUT_CHANNELS: u32 = 2;
    const DEFAULT_OUTPUT_CHANNELS: u32 = 2;

    const DEFAULT_AUX_INPUTS: Option<AuxiliaryIOConfig> = None;
    const DEFAULT_AUX_OUTPUTS: Option<AuxiliaryIOConfig> = None;

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(), 
            self.params.editor_state.clone()
        )
    }

    fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        // This works with any symmetrical IO layout
        config.num_input_channels == config.num_output_channels && config.num_input_channels > 0
    }

    fn initialize(
        &mut self,
        _bus_config: &BusConfig,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let clip = self.params.clip.smoothed.next();
            let gain = match self.params.autogain.value() {
                false => self.params.gain.smoothed.next(),
                true => {
                    let db = util::gain_to_db(clip);
                    let reduce = util::db_to_gain(-db);
                    reduce.clamp(0.0, util::db_to_gain(self.CLIPPING_FAC))
                }
            };
            let dist_type = self.params.dist_type.value();
            let clipping_fac = self.CLIPPING_FAC;
            let wet_mix = self.params.mix.smoothed.next();
            let dry_mix = 1.0 - wet_mix;

            for sample in channel_samples {
                let dry_sample = *sample;
                *sample *= clip;

                match dist_type{
                    DistTypes::HardClip => *sample = sample.clamp(-util::db_to_gain(clipping_fac), util::db_to_gain(clipping_fac)),
                    // DistTypes::SoftClip => todo!(),
                    // DistTypes::SineFold => todo!(),
                    DistTypes::Saturate => *sample = sample.tanh(),
                    DistTypes::SineFold => *sample = sample.sin(),
                    _ => {}
                }
                
                *sample *= gain;
                let wet_sample = *sample;
                *sample = dry_sample * dry_mix;
                *sample += wet_sample * wet_mix;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for YasYas {
    const CLAP_ID: &'static str = "com.consulthp.beanstortion";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("hea");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for YasYas {
    const VST3_CLASS_ID: [u8; 16] = *b"COOLpluginmoment";

    // And don't forget to change these categories, see the docstring on `VST3_CATEGORIES` for more
    // information
    const VST3_CATEGORIES: &'static str = "Fx|Distortion";
}

nih_export_clap!(YasYas);
nih_export_vst3!(YasYas);

#[derive(PartialEq, Eq, Enum, Clone, Copy, Debug)]
enum DistTypes {
    HardClip,
    SoftClip,
    SineFold,
    Saturate
}

impl DistTypes {
    const ALL: [DistTypes; 4] = [
        DistTypes::HardClip,
        DistTypes::SoftClip,
        DistTypes::SineFold,
        DistTypes::Saturate
    ];
}

impl Default for DistTypes {
    fn default() -> Self {
        DistTypes::HardClip
    }
}

impl Display for DistTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}