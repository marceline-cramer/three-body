use std::{f32::consts::TAU, fs::File, io::Write};

use glam::DVec2;
use pretty::RcDoc;
use raqote::{DrawOptions, DrawTarget, PathBuilder, SolidSource, Transform};
use rayon::{
    iter::{ParallelBridge, ParallelIterator},
    slice::ParallelSlice,
};
use rustfft::{FftPlanner, num_complex::Complex64};
use serde::{Deserialize, Serialize};

fn main() {
    let config: Config = toml::from_str(&std::fs::read_to_string("orbits.toml").unwrap()).unwrap();

    let sim_config = SimulationConfig {
        frames: 140,
        subframes: 100,
    };

    // bake each orbit in the config in parallel
    let orbits: Vec<_> = config
        .orbit
        .iter()
        .par_bridge()
        .map(|orbit| bake(&sim_config, orbit))
        .collect();

    let doc = RcDoc::text("orbits = ").append(
        pretty_elm_list(orbits.iter().map(BakedOrbit::to_doc))
            .nest(2)
            .group(),
    );

    let mut stdout = std::io::stdout();
    let template = include_str!("OrbitsTemplate.elm");
    stdout.write_all(template.as_bytes()).unwrap();
    doc.render(80, &mut stdout).unwrap();
    stdout.flush().unwrap();
}

pub fn bake(sim_config: &SimulationConfig, orbit_config: &OrbitConfig) -> BakedOrbit {
    let orbit = orbit_config.to_orbit();

    let simulated = simulate_closed(sim_config, &orbit);

    let mut baked_bodies = analyze(&simulated);

    baked_bodies
        .iter_mut()
        .for_each(|body| body.optimize(0.001));

    let by_body = transpose(&simulated, Clone::clone);

    let total_frames = simulated.len();
    let mut compressed = Vec::new();
    for (body, baseline) in baked_bodies.iter().zip(by_body.iter()) {
        let positions = inverse_analyze(total_frames, body);
        eprintln!("optimization error: {}", rms_error(&positions, baseline));
        compressed.push(positions);
    }

    let compressed = transpose(&compressed, Clone::clone);
    render(sim_config, &orbit, &compressed);

    BakedOrbit {
        name: orbit_config.name.clone(),
        period: orbit_config.period,
        energy: orbit_config.energy,
        bodies: baked_bodies.clone(),
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub orbit: Vec<OrbitConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrbitConfig {
    pub name: String,
    pub period: f64,
    pub energy: f64,
    pub masses: Vec<f64>,
    pub positions: Vec<DVec2>,
    pub velocities: Vec<DVec2>,
}

impl OrbitConfig {
    pub fn to_orbit(&self) -> Orbit {
        let colors = [
            SolidSource::from_unpremultiplied_argb(255, 255, 0, 0),
            SolidSource::from_unpremultiplied_argb(255, 0, 255, 0),
            SolidSource::from_unpremultiplied_argb(255, 0, 0, 255),
        ];

        let bodies = self
            .masses
            .iter()
            .zip(self.positions.iter())
            .zip(self.velocities.iter())
            .zip(colors.iter().cycle())
            .map(|(((mass, position), velocity), color)| Body {
                mass: *mass,
                color: *color,
                position: *position,
                velocity: *velocity,
            })
            .collect();

        Orbit {
            name: self.name.clone(),
            initial_conds: bodies,
            period: self.period,
        }
    }
}

pub fn analyze(positions: &[Vec<DVec2>]) -> Vec<BakedBody> {
    let mut planner = FftPlanner::new();
    let frame_num = positions.len();
    let fft = planner.plan_fft_forward(frame_num);

    let mut freqs = transpose(positions, |pos| Complex64 {
        re: pos.x,
        im: pos.y,
    });

    freqs
        .iter_mut()
        .par_bridge()
        .for_each(|body| fft.process(body));

    freqs
        .into_iter()
        .map(|frames| {
            frames
                .into_iter()
                .enumerate()
                .map(|(idx, freq)| fft_to_freq(idx, freq, frame_num))
                .collect()
        })
        .map(|frequencies| BakedBody { frequencies })
        .collect()
}

pub fn fft_to_freq(idx: usize, fft: Complex64, frame_num: usize) -> FrequencyComponent {
    let half = frame_num / 2;

    let freq = if idx == 0 {
        0.0
    } else if idx < half {
        -(idx as f64)
    } else {
        (frame_num as f64) - (idx as f64)
    };

    FrequencyComponent {
        freq,
        amplitude: (fft.im * fft.im + fft.re * fft.re).sqrt() / frame_num as f64,
        phase: (-fft.im).atan2(fft.re),
    }
}

pub fn inverse_analyze(frames: usize, body: &BakedBody) -> Vec<DVec2> {
    (0..frames)
        .map(|idx| {
            let sample = (idx as f64) / (frames as f64);
            body.frequencies
                .iter()
                .map(|freq| freq.sample(sample))
                .sum()
        })
        .collect()
}

pub struct BakedOrbit {
    pub name: String,
    pub period: f64,
    pub energy: f64,
    pub bodies: Vec<BakedBody>,
}

impl BakedOrbit {
    pub fn to_doc(&self) -> RcDoc<()> {
        let name = RcDoc::text("\"").append(self.name.clone()).append("\"");
        let bodies = pretty_elm_list(self.bodies.iter().map(BakedBody::to_doc));

        pretty_elm_struct([
            ("name", name),
            ("period", RcDoc::text(self.period.to_string())),
            ("energy", RcDoc::text(self.energy.to_string())),
            ("bodies", bodies),
        ])
    }
}

#[derive(Clone, Debug)]
pub struct BakedBody {
    pub frequencies: Vec<FrequencyComponent>,
}

impl BakedBody {
    pub fn to_doc(&self) -> RcDoc<()> {
        pretty_elm_struct([(
            "frequencies",
            pretty_elm_list(self.frequencies.iter().map(FrequencyComponent::to_doc)),
        )])
    }

    pub fn optimize(&mut self, cutoff: f64) {
        let original_length = self.frequencies.len();
        self.frequencies.retain(|freq| freq.amplitude > cutoff);
        let new_length = self.frequencies.len();
        eprintln!("optimized #freqs from {original_length} to {new_length}",);
    }
}

#[derive(Clone, Debug)]
pub struct FrequencyComponent {
    pub freq: f64,
    pub amplitude: f64,
    pub phase: f64,
}

impl FrequencyComponent {
    pub const ZERO: Self = Self {
        freq: 0.0,
        amplitude: 0.0,
        phase: 0.0,
    };

    pub fn sample(&self, at: f64) -> DVec2 {
        let theta = std::f64::consts::TAU * at * self.freq + self.phase;
        DVec2::from_angle(-theta) * self.amplitude
    }

    pub fn to_doc(&self) -> RcDoc<()> {
        let fmt_real = |val: f64| {
            let precision = 100000000.0;
            RcDoc::text(((val * precision).round() / precision).to_string())
        };

        pretty_elm_struct([
            ("freq", fmt_real(self.freq)),
            ("amplitude", fmt_real(self.amplitude)),
            ("phase", fmt_real(self.phase)),
        ])
    }
}

pub fn pretty_elm_struct<'a>(
    fields: impl IntoIterator<Item = (&'static str, RcDoc<'a, ()>)>,
) -> RcDoc<'a, ()> {
    let inner = fields
        .into_iter()
        .map(|(name, value)| RcDoc::text(name).append(" = ").append(value));

    RcDoc::text("{").append(pretty_elm_group(inner)).append("}")
}

pub fn pretty_elm_list<'a>(items: impl IntoIterator<Item = RcDoc<'a, ()>>) -> RcDoc<'a, ()> {
    RcDoc::text("[").append(pretty_elm_group(items)).append("]")
}

pub fn pretty_elm_group<'a>(items: impl IntoIterator<Item = RcDoc<'a, ()>>) -> RcDoc<'a, ()> {
    let items = items.into_iter().map(|doc| doc.nest(2));
    let separator = RcDoc::hardline().flat_alt(RcDoc::nil()).append(", ");

    RcDoc::space()
        .append(RcDoc::intersperse(items, separator))
        .append(RcDoc::line())
        .group()
}

pub fn simulate_closed(config: &SimulationConfig, orbit: &Orbit) -> Vec<Vec<DVec2>> {
    let mut reversed = orbit.clone();

    for body in reversed.initial_conds.iter_mut() {
        body.velocity = -body.velocity;
    }

    let (mut forwards, mut backwards) =
        rayon::join(|| simulate(config, orbit), || simulate(config, &reversed));

    backwards.reverse();

    // simulations should end where they started
    // remove the last element to even out the period
    forwards.pop();
    backwards.pop();

    let forwards_error = transpose(&forwards, Clone::clone);
    let backwards_error = transpose(&backwards, Clone::clone);

    for (forwards, backwards) in forwards_error.iter().zip(backwards_error.iter()) {
        let error = rms_error(forwards, backwards);
        eprintln!("closed simulation RMS error: {error}",);
        assert!(error < 0.001);
    }

    let frame_num = forwards.len();
    for (idx, (forwards, backwards)) in forwards.iter_mut().zip(backwards.iter()).enumerate() {
        let position = (idx as f64) / (frame_num as f64);
        let blend = position;

        for (forward, backward) in forwards.iter_mut().zip(backwards.iter()) {
            *forward = forward.lerp(*backward, blend);
        }
    }

    forwards
}

pub fn simulate(config: &SimulationConfig, orbit: &Orbit) -> Vec<Vec<DVec2>> {
    let frame_num = config.frames * config.subframes;
    let timestep = orbit.period / frame_num as f64;
    let mut bodies = orbit.initial_conds.clone();
    let mut history = Vec::with_capacity(frame_num);

    let first: Vec<_> = bodies.iter().map(|body| body.position).collect();
    history.push(first.clone());

    let mut last = vec![];
    for _ in 0..frame_num {
        for _ in 0..10000 {
            step(timestep / 10000.0, &mut bodies);
        }

        last = bodies.iter().map(|body| body.position).collect();
        history.push(last.clone())
    }

    eprintln!("start-end simulation drift: {}", rms_error(&first, &last));

    history
}

pub fn rms_error(lhs: &[DVec2], rhs: &[DVec2]) -> f64 {
    lhs.iter()
        .zip(rhs.iter())
        .map(|(lhs_pos, rhs_pos)| lhs_pos.distance_squared(*rhs_pos) / lhs.len() as f64)
        .sum()
}

pub fn transpose<T, O: Clone>(positions: &[Vec<T>], map: impl Fn(&T) -> O) -> Vec<Vec<O>> {
    let mut by_body = vec![Vec::new(); positions[0].len()];

    for frame in positions.iter() {
        for (by_body, position) in by_body.iter_mut().zip(frame.iter()) {
            by_body.push(map(position));
        }
    }

    by_body
}

pub fn render(config: &SimulationConfig, orbit: &Orbit, positions: &[Vec<DVec2>]) {
    let width = 400;
    let height = 400;

    let frames: Vec<_> = positions
        .par_chunks(config.subframes)
        .map(|frame| {
            let mut dt = DrawTarget::new(width as _, height as _);
            dt.clear(SolidSource::from_unpremultiplied_argb(255, 100, 100, 100));
            dt.set_transform(&Transform::scale(50.0, 50.0).then_translate((200.0, 200.0).into()));

            let step = frame.len().div_ceil(10);
            for subframe in frame.iter().step_by(step) {
                draw(&mut dt, &orbit.initial_conds, subframe);
            }

            let mut frame = gif::Frame::from_rgba(width, height, dt.get_data_u8_mut());
            frame.delay = 2;
            frame.dispose = gif::DisposalMethod::Keep;
            frame.make_lzw_pre_encoded();

            frame
        })
        .collect();

    let path = format!("target/{}.gif", orbit.name);
    let mut image = File::create(&path).unwrap();
    let mut encoder = gif::Encoder::new(&mut image, width, height, &[]).unwrap();
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();

    for frame in frames {
        encoder.write_lzw_pre_encoded_frame(&frame).unwrap();
    }
}

#[derive(Clone, Debug)]
pub struct SimulationConfig {
    pub frames: usize,
    pub subframes: usize,
}

#[derive(Clone, Debug)]
pub struct Orbit {
    pub name: String,
    pub initial_conds: Vec<Body>,
    pub period: f64,
}

pub fn draw(dt: &mut DrawTarget, bodies: &[Body], positions: &[DVec2]) {
    for (body, position) in bodies.iter().zip(positions.iter()) {
        let mut pb = PathBuilder::new();
        pb.arc(position.x as f32, position.y as f32, 0.08, 0.0, TAU);

        let color =
            SolidSource::from_unpremultiplied_argb(13, body.color.r, body.color.g, body.color.b);

        let path = pb.finish();
        let src = raqote::Source::Solid(color);

        let options = DrawOptions::new();
        dt.fill(&path, &src, &options);
    }
}

pub fn step(dt: f64, bodies: &mut [Body]) {
    apply_forces(dt, bodies);
    update(dt, bodies);
}

pub fn apply_forces(dt: f64, bodies: &mut [Body]) {
    for body in 0..bodies.len() {
        for other in (body + 1)..bodies.len() {
            let [body, other] = bodies.get_disjoint_mut([body, other]).unwrap();

            let delta = body.position - other.position;

            let mass = body.mass * other.mass;
            let r2 = delta.length_squared();
            let force = mass / r2;

            let delta = delta.normalize();
            body.velocity -= dt * force * delta;
            other.velocity += dt * force * delta;
        }
    }
}

pub fn update(dt: f64, bodies: &mut [Body]) {
    for body in bodies.iter_mut() {
        body.position += body.velocity * dt;
    }
}

#[derive(Clone, Debug)]
pub struct Body {
    pub mass: f64,
    pub color: SolidSource,
    pub position: DVec2,
    pub velocity: DVec2,
}
