use nih_plug::prelude::{Editor};
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState};
use std::sync::Arc;

use crate::{YasYasParams};

/// VIZIA uses points instead of pixels for text
const POINT_SCALE: f32 = 0.75;

const STYLE: &str = r#""#;

#[derive(Lens)]
struct Data {
    params: Arc<YasYasParams>,
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::from_size(200, 350)
}

pub(crate) fn create(
    params: Arc<YasYasParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, move |cx, _| {
        cx.add_theme(STYLE);

        Data {
            params: params.clone(),
        }
        .build(cx);

        ResizeHandle::new(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "beanstortion")
                .font(assets::NOTO_SANS_BOLD)
                .font_size(40.0 * POINT_SCALE)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));

            // NOTE: VIZIA adds 1 pixel of additional height to these labels, so we'll need to
            //       compensate for that
            Label::new(cx, "beanify").bottom(Pixels(-1.0));
            ParamSlider::new(cx, Data::params, |params| &params.clip);
            Label::new(cx, "gain").bottom(Pixels(-1.0));
            ParamSlider::new(cx, Data::params, |params| &params.gain);
            Label::new(cx, "mix").bottom(Pixels(-1.0));
            ParamSlider::new(cx, Data::params, |params| &params.mix);
            Label::new(cx, "type");
            ParamSlider::new(cx, Data::params, |params| &params.dist_type);
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));
    })
}