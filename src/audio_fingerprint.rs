use std::sync::Arc;

use super::*;
use tokio::sync::Notify;

#[derive(Clone, Debug)]
pub struct AudioFingerPrint {
    pub hash: Option<f32>,
    pub check_audio_formats: Option<CheckAudioFormats>,
}
impl AudioFingerPrint {
    pub async fn new(window: &Window) -> Option<Self> {
        Some(Self {
            hash: audio_hash().await,
            check_audio_formats: CheckAudioFormats::new(&window),
        })
    }
}

pub async fn audio_hash() -> Option<f32> {
    let sample_rate = 44000.;
    let audio_context = OfflineAudioContext::new_with_context_options(
        &OfflineAudioContextOptions::new(5000, sample_rate),
    )
    .unwrap();
    let oscillator = audio_context.create_oscillator().ok()?;
    oscillator.set_type(web_sys::OscillatorType::Triangle);
    oscillator.frequency().set_value(1000.);
    let compressor = audio_context.create_dynamics_compressor().ok()?;
    compressor.threshold().set_value(-50.);
    compressor.knee().set_value(40.);
    compressor.ratio().set_value(12.);
    compressor.attack().set_value(0.20);
    compressor.release().set_value(0.02);
    oscillator
        .connect_with_audio_node(compressor.as_ref())
        .ok()?;
    compressor
        .connect_with_audio_node(audio_context.destination().as_ref())
        .ok()?;

    let hash = Rc::new(RefCell::new(0.));
    let hash_c = Rc::clone(&hash);
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();
    let f = Closure::once_into_js(move |event: OfflineAudioCompletionEvent| {
        if let Some(channel_data) = event.rendered_buffer().get_channel_data(0).ok() {
            for sample in channel_data {
                *hash_c.borrow_mut() += sample.abs();
            }
        }
        notify2.notify_one();
    })
    .dyn_into::<Function>()
    .ok()?;

    audio_context.set_oncomplete(Some(&f));
    oscillator.start().ok()?;

    JsFuture::from(audio_context.start_rendering().ok()?)
        .await
        .ok()?;
    notify.notified().await;

    Some(hash.take())
}

#[derive(Clone, Debug)]
pub struct CheckAudioFormats {
    pub audio_aac_probably: bool,
    pub audio_flac_probably: bool,
    pub audio_mpeg_probably: bool,
    pub audio_ogg_flac_probably: bool,
    pub audio_ogg_vorbis_probably: bool,
    pub audio_ogg_opus_probably: bool,
    pub audio_wav_probably: bool,
    pub audio_webm_vorbis_probably: bool,
    pub audio_webm_opus_probably: bool,
    pub audio_mp4_probably: bool,
}

impl CheckAudioFormats {
    pub fn new(window: &Window) -> Option<Self> {
        let audio = window
            .document()?
            .create_element("audio")
            .ok()?
            .dyn_into::<web_sys::HtmlMediaElement>()
            .ok()?;

        Some(Self {
            audio_aac_probably: audio.can_play_type("audio/aac").contains("probably"),
            audio_flac_probably: audio.can_play_type("audio/flac").contains("probably"),
            audio_mpeg_probably: audio.can_play_type("audio/mpeg").contains("probably"),
            audio_ogg_flac_probably: audio
                .can_play_type(r#"audio/ogg; codecs="flac""#)
                .contains("probably"),
            audio_ogg_vorbis_probably: audio
                .can_play_type(r#"audio/ogg; codecs="vorbis""#)
                .contains("probably"),
            audio_ogg_opus_probably: audio
                .can_play_type(r#"audio/ogg; codecs="opus""#)
                .contains("probably"),
            audio_wav_probably: audio
                .can_play_type(r#"audio/wav; codecs="1""#)
                .contains("probably"),
            audio_webm_vorbis_probably: audio
                .can_play_type(r#"audio/webm; codecs="vorbis""#)
                .contains("probably"),
            audio_webm_opus_probably: audio
                .can_play_type(r#"audio/webm; codecs="opus""#)
                .contains("probably"),
            audio_mp4_probably: audio.can_play_type(r#"audio/mp4"#).contains("probably"),
        })
    }
}
