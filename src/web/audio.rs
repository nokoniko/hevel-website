//! Gapless looping player on the Web Audio API.
//!
//! The file is decoded once, so starting is instant and the loop point has no gap (an `<audio loop>`
//! element restarts with a small pause). Pausing remembers the position. Loops are scheduled as
//! back-to-back sources with a short equal-power crossfade so the join never clicks or pops.
use super::dom::document;
use futures::{future::LocalBoxFuture, future::Shared, FutureExt};
use gloo_events::EventListener;
use gloo_timers::callback::Timeout;
use js_sys::Uint8Array;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, GainNode};

const REPEATS: usize = 100; // ~70 minutes of this track
const CROSSFADE: f64 = 0.05; // seconds
const CURVE_POINTS: usize = 32;
/// Track state: the song starts quiet and fades smoothly up to its full volume.
const FADE_FROM: f32 = 0.2;
const FADE_TO: f32 = 0.4;
const FADE_SECONDS: f64 = 8.0;

fn curve(f: impl Fn(f64) -> f64) -> Vec<f32> {
    (0..CURVE_POINTS)
        .map(|i| f(i as f64 / (CURVE_POINTS - 1) as f64 * std::f64::consts::FRAC_PI_2) as f32)
        .collect()
}

struct Session {
    out: GainNode,
    nodes: Vec<(AudioBufferSourceNode, GainNode)>,
    t0: f64,
    first: f64,
}

struct Inner {
    ctx: AudioContext,
    buffer: Option<AudioBuffer>,
    session: Option<Session>,
    offset: f64,
    wanted: bool,
    want_music: bool,
    volume: f32,
    /// Fade up to this volume over this many seconds once playback actually starts.
    fade: Option<(f32, f64)>,
    on_change: Rc<dyn Fn(bool)>,
    visibility: Option<EventListener>,
}

#[derive(Clone)]
pub struct Player {
    inner: Rc<RefCell<Inner>>,
    loaded: Shared<LocalBoxFuture<'static, ()>>,
}

async fn load(ctx: &AudioContext, url: &str) -> Option<AudioBuffer> {
    let bytes = gloo_net::http::Request::get(url).send().await.ok()?.binary().await.ok()?;
    let promise = ctx.decode_audio_data(&Uint8Array::from(bytes.as_slice()).buffer()).ok()?;
    JsFuture::from(promise).await.ok()?.dyn_into().ok()
}

impl Player {
    pub fn new(url: &str, volume: f32, on_change: impl Fn(bool) + 'static) -> Self {
        let ctx = AudioContext::new().expect("AudioContext");
        let inner = Rc::new(RefCell::new(Inner {
            ctx: ctx.clone(),
            buffer: None,
            session: None,
            offset: 0.0,
            wanted: false,
            want_music: false,
            volume,
            fade: None,
            on_change: Rc::new(on_change),
            visibility: None,
        }));

        let loaded = {
            let inner = inner.clone();
            let url = url.to_string();
            async move {
                let buffer = load(&ctx, &url).await;
                inner.borrow_mut().buffer = buffer;
            }
            .boxed_local()
            .shared()
        };
        spawn_local(loaded.clone());

        let player = Player { inner: inner.clone(), loaded };

        // Only play while the page is visible, and only if the Track state is still selected.
        let weak = Rc::downgrade(&inner);
        let loaded = player.loaded.clone();
        let listener = EventListener::new(&document(), "visibilitychange", move |_| {
            let Some(inner) = weak.upgrade() else { return };
            let p = Player { inner, loaded: loaded.clone() };
            if document().hidden() {
                p.pause();
            } else if p.inner.borrow().want_music {
                p.play();
            }
        });
        inner.borrow_mut().visibility = Some(listener);
        player
    }

    /// Call inside a click so the browser lets audio start later without a gesture.
    pub fn unlock(&self) {
        let _ = self.inner.borrow().ctx.resume();
    }

    pub fn play(&self) {
        self.inner.borrow_mut().wanted = true;
        let this = self.clone();
        spawn_local(async move {
            this.loaded.clone().await;
            let resume = this.inner.borrow().ctx.resume();
            if let Ok(promise) = resume {
                let _ = JsFuture::from(promise).await;
            }
            let _ = this.start();
        });
    }

    fn start(&self) -> Result<(), JsValue> {
        let mut s = self.inner.borrow_mut();
        let Some(buf) = s.buffer.clone() else { return Ok(()) };
        if !s.wanted || s.session.is_some() {
            return Ok(());
        }

        let ctx = s.ctx.clone();
        let dur = buf.duration();
        let period = dur - CROSSFADE;
        let t0 = ctx.current_time() + 0.03;
        let first = s.offset.min(dur - CROSSFADE * 2.0);

        let out = ctx.create_gain()?;
        out.gain().set_value(s.volume);
        out.connect_with_audio_node(&ctx.destination())?;

        let fade_in = curve(f64::sin);
        let fade_out = curve(f64::cos);
        let mut nodes = Vec::with_capacity(REPEATS + 1);
        for i in 0..=REPEATS {
            let src = ctx.create_buffer_source()?;
            src.set_buffer(Some(&buf));
            let gain = ctx.create_gain()?;
            gain.gain().set_value(0.0);
            src.connect_with_audio_node(&gain)?;
            gain.connect_with_audio_node(&out)?;

            // Source 0 resumes mid-file (short fade-in); the rest start at 0 and crossfade with the previous one.
            let start = if i == 0 { t0 } else { t0 + (dur - first - CROSSFADE) + (i - 1) as f64 * period };
            let fade_in_len = if i == 0 { 0.01 } else { CROSSFADE };
            src.start_with_when_and_grain_offset(start, if i == 0 { first } else { 0.0 })?;
            gain.gain().set_value_curve_at_time(&mut fade_in.clone(), start, fade_in_len)?;
            let length = if i == 0 { dur - first } else { dur };
            gain.gain().set_value_curve_at_time(&mut fade_out.clone(), start + length - CROSSFADE, CROSSFADE)?;
            nodes.push((src, gain));
        }

        // Smooth, perceptually even fade (exponential) from the start volume up to the target.
        if let Some((target, seconds)) = s.fade.take() {
            let gain = out.gain();
            gain.set_value_at_time(s.volume, t0)?;
            gain.exponential_ramp_to_value_at_time(target, t0 + seconds)?;
            s.volume = target; // later resumes continue at the full volume
        }

        s.session = Some(Session { out, nodes, t0, first });
        let on_change = s.on_change.clone();
        drop(s);
        on_change(true);
        Ok(())
    }

    pub fn pause(&self) {
        let mut s = self.inner.borrow_mut();
        s.wanted = false;
        let (Some(session), Some(buf)) = (s.session.take(), s.buffer.clone()) else { return };

        // Remember where we were: the first source plays `first → end`, later ones restart at 0 every `period`.
        let elapsed = (s.ctx.current_time() - session.t0).max(0.0);
        let head = buf.duration() - session.first - CROSSFADE;
        s.offset = if elapsed < head { session.first + elapsed } else { (elapsed - head) % (buf.duration() - CROSSFADE) };

        stop_session(&s.ctx, session);
        let on_change = s.on_change.clone();
        drop(s);
        on_change(false);
    }

    /// Change the volume, smoothly over `seconds` (also applies to the next play()).
    pub fn set_volume(&self, volume: f32, seconds: f64) {
        let mut s = self.inner.borrow_mut();
        s.volume = volume;
        let Some(session) = &s.session else { return };
        let now = s.ctx.current_time();
        let gain = session.out.gain();
        let _ = gain.cancel_scheduled_values(now);
        let _ = gain.set_value_at_time(gain.value(), now);
        if seconds > 0.0 {
            let _ = gain.exponential_ramp_to_value_at_time(volume, now + seconds);
        } else {
            let _ = gain.set_value_at_time(volume, now);
        }
    }

    /// Track state: play, fading smoothly up to full volume. Any other state: pause.
    pub fn select(&self, is_music: bool) {
        {
            let mut s = self.inner.borrow_mut();
            s.want_music = is_music;
            s.fade = None;
        }
        if !is_music {
            self.pause();
            return;
        }
        {
            let mut s = self.inner.borrow_mut();
            s.volume = FADE_FROM;
            s.fade = Some((FADE_TO, FADE_SECONDS));
        }
        // If it is already playing (e.g. re-selecting Track), fade from the current level instead.
        if self.inner.borrow().session.is_some() {
            self.set_volume(FADE_TO, FADE_SECONDS);
            self.inner.borrow_mut().fade = None;
        } else {
            self.play();
        }
    }

    pub fn dispose(&self) {
        let mut s = self.inner.borrow_mut();
        s.wanted = false;
        s.want_music = false;
        s.fade = None;
        s.visibility = None;
        if let Some(session) = s.session.take() {
            stop_session(&s.ctx, session);
        }
        let ctx = s.ctx.clone();
        Timeout::new(150, move || {
            let _ = ctx.close();
        })
        .forget();
    }
}

#[allow(deprecated)] // web-sys marks the (still standard) stop() as unstable-API
/// Fade out quickly instead of cutting, then tear the graph down.
fn stop_session(ctx: &AudioContext, session: Session) {
    let now = ctx.current_time();
    let gain = session.out.gain();
    let _ = gain.cancel_scheduled_values(now);
    let _ = gain.set_value_at_time(gain.value(), now);
    let _ = gain.linear_ramp_to_value_at_time(0.0, now + 0.04);
    Timeout::new(80, move || {
        for (src, gain) in &session.nodes {
            let _ = src.stop(); // errors if already stopped
            let _ = src.disconnect();
            let _ = gain.disconnect();
        }
        let _ = session.out.disconnect();
    })
    .forget();
}
