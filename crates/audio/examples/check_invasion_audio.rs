//! Exercise the game's PCM decoder and Bevy audio sink without a renderer,
//! game window, match, network listener or input injection.
use bevy::{
    app::ScheduleRunnerPlugin,
    audio::{AddAudioSource, AudioSinkPlayback, Decodable, Source, Volume},
    prelude::*,
};
#[derive(Resource)]
struct Check {
    started: std::time::Instant,
    reported: bool,
    samples: usize,
}
fn main() {
    let file = std::env::args().nth(1).expect("OGG or WAV sample path");
    let bytes = std::fs::read(file).expect("sample bytes");
    let pcm = audio::decode_audio_bytes(&bytes).expect("game PCM decoder");
    let rate = pcm.decoder().sample_rate().get();
    let channels = pcm.decoder().channels().get();
    let decoded: Vec<f32> = pcm.decoder().collect();
    let peak = decoded.iter().fold(0.0f32, |a, v| a.max(v.abs()));
    assert!(
        peak > 0.001 && decoded.iter().all(|v| v.is_finite()),
        "silent/invalid decoded audio"
    );
    println!(
        "PCM PASS: samples={} channels={channels} rate={rate} peak={peak:.3}",
        decoded.len()
    );
    let mut app = App::new();
    app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    )))
    .add_plugins(bevy::asset::AssetPlugin::default())
    .add_plugins(bevy::audio::AudioPlugin::default())
    .add_audio_source::<audio::PcmAudio>();
    let handle = app
        .world_mut()
        .resource_mut::<Assets<audio::PcmAudio>>()
        .add(pcm.with_live_pan());
    app.world_mut().spawn((
        AudioPlayer(handle),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(0.7)),
    ));
    app.insert_resource(Check {
        started: std::time::Instant::now(),
        reported: false,
        samples: decoded.len(),
    })
    .add_systems(Update, check);
    app.run();
}
fn check(mut state: ResMut<Check>, sinks: Query<&AudioSink>, mut exit: MessageWriter<AppExit>) {
    if !state.reported {
        if let Some(sink) = sinks.iter().next() {
            println!(
                "SINK PASS: paused={} empty={} samples={}",
                sink.is_paused(),
                sink.empty(),
                state.samples
            );
            state.reported = true;
        }
    }
    if state.started.elapsed().as_secs_f32() > 3.0 {
        if !state.reported {
            eprintln!("FAIL: no audio output sink was created");
            exit.write(AppExit::error());
        } else {
            exit.write(AppExit::Success);
        }
    }
}
