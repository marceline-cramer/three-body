use std::{f32::consts::TAU, fs::File};

use glam::{DVec2, dvec2};
use raqote::{DrawOptions, DrawTarget, PathBuilder, SolidSource, Transform};
use rayon::{
    iter::{ParallelBridge, ParallelIterator},
    slice::ParallelSlice,
};
use rustfft::{FftPlanner, num_complex::Complex64};

fn main() {
    let orbit = Orbit {
        period: 6.325897,
        initial_conds: vec![
            Body {
                mass: 1.0,
                position: dvec2(-1.0, 0.0),
                velocity: dvec2(0.347113, 0.532727),
                color: SolidSource::from_unpremultiplied_argb(255, 255, 0, 0),
            },
            Body {
                mass: 1.0,
                position: dvec2(1.0, 0.0),
                velocity: dvec2(0.347113, 0.532727),
                color: SolidSource::from_unpremultiplied_argb(255, 0, 255, 0),
            },
            Body {
                mass: 1.0,
                position: dvec2(0.0, 0.0),
                velocity: dvec2(-0.694226, -1.065454),
                color: SolidSource::from_unpremultiplied_argb(255, 0, 0, 255),
            },
        ],
    };

    let config = SimulationConfig {
        frames: 100,
        subframes: 10,
    };

    let simulated = simulate_closed(&config, &orbit);

    let mut freqs = analyze(&simulated);
    freqs.iter_mut().for_each(|freqs| optimize(0.01, freqs));
    eprintln!("{freqs:#?}");

    let by_body = invert(&simulated, Clone::clone);

    let total_frames = simulated.len();
    let mut compressed = Vec::new();
    for (body_freqs, baseline) in freqs.iter().zip(by_body.iter()) {
        let positions = inverse_analyze(total_frames, body_freqs);
        println!("{}", rms_error(&positions, baseline));
        compressed.push(positions);
    }

    let compressed = invert(&compressed, Clone::clone);
    render(&config, &orbit, &compressed);
}

pub fn invert<T, O: Clone>(positions: &[Vec<T>], map: impl Fn(&T) -> O) -> Vec<Vec<O>> {
    let mut by_body = vec![Vec::new(); positions[0].len()];

    for frame in positions.iter() {
        for (by_body, position) in by_body.iter_mut().zip(frame.iter()) {
            by_body.push(map(position));
        }
    }

    by_body
}

pub fn analyze(positions: &[Vec<DVec2>]) -> Vec<Vec<FrequencyComponent>> {
    let mut planner = FftPlanner::new();
    let frame_num = positions.len();
    let fft = planner.plan_fft_forward(frame_num);

    let mut freqs = invert(positions, |pos| Complex64 {
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
        .collect()
}

pub fn fft_to_freq(idx: usize, fft: Complex64, frame_num: usize) -> FrequencyComponent {
    let half = frame_num / 2;

    let freq = if idx == 0 {
        0.0
    } else if idx < half {
        idx as f64
    } else {
        (idx as f64) - (frame_num as f64)
    };

    FrequencyComponent {
        freq: -freq,
        amplitude: (fft.im * fft.im + fft.re * fft.re).sqrt() / frame_num as f64,
        phase: (-fft.im).atan2(fft.re),
    }
}

pub fn optimize(cutoff: f64, freqs: &mut Vec<FrequencyComponent>) {
    let dc = freqs.iter().find(|freq| freq.freq.abs() < 0.001).cloned();
    freqs.retain(|freq| freq.amplitude > cutoff);
    freqs.extend(dc);
}

pub fn inverse_analyze(frames: usize, freqs: &[FrequencyComponent]) -> Vec<DVec2> {
    (0..frames)
        .map(|idx| {
            let sample = (idx as f64) / (frames as f64);
            freqs.iter().map(|freq| freq.sample(sample)).sum()
        })
        .collect()
}

pub fn rms_error(lhs: &[DVec2], rhs: &[DVec2]) -> f64 {
    lhs.iter()
        .zip(rhs.iter())
        .map(|(lhs_pos, rhs_pos)| lhs_pos.distance_squared(*rhs_pos) / lhs.len() as f64)
        .sum()
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
}

pub fn render(config: &SimulationConfig, orbit: &Orbit, positions: &[Vec<DVec2>]) {
    let subalpha = (255 / config.subframes / 2) as u8;
    let width = 400;
    let height = 400;

    let frames: Vec<_> = positions
        .chunks(config.subframes)
        .collect::<Vec<_>>()
        .par_windows(2)
        .map(|frames| {
            let mut dt = DrawTarget::new(width as _, height as _);
            dt.set_transform(&Transform::scale(100.0, 100.0).then_translate((200.0, 200.0).into()));

            for frame in frames.iter() {
                for subframe in frame.iter() {
                    draw(&mut dt, &orbit.initial_conds, subframe, subalpha);
                }
            }

            let mut frame = gif::Frame::from_rgba(width, height, dt.get_data_u8_mut());
            frame.delay = 2;
            frame.dispose = gif::DisposalMethod::Background;
            frame.make_lzw_pre_encoded();

            frame
        })
        .collect();

    let mut image = File::create("target/orbit.gif").unwrap();
    let mut encoder = gif::Encoder::new(&mut image, width, height, &[]).unwrap();
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();

    for frame in frames {
        encoder.write_lzw_pre_encoded_frame(&frame).unwrap();
    }
}

pub fn simulate_closed(config: &SimulationConfig, orbit: &Orbit) -> Vec<Vec<DVec2>> {
    let mut reversed = orbit.clone();

    for body in reversed.initial_conds.iter_mut() {
        body.velocity = -body.velocity;
    }

    let (mut forwards, backwards) =
        rayon::join(|| simulate(config, orbit), || simulate(config, &reversed));

    let frame_num = forwards.len();
    for (idx, frame) in forwards.iter_mut().enumerate() {
        let position = (idx as f64) / (frame_num as f64);
        let blend = position;

        let backwards_idx = frame_num - idx - 1;
        let backwards_frame = &backwards[backwards_idx];

        for (forward, backward) in frame.iter_mut().zip(backwards_frame.iter()) {
            *forward = forward.lerp(*backward, blend);
        }
    }

    forwards
}

pub fn simulate(config: &SimulationConfig, orbit: &Orbit) -> Vec<Vec<DVec2>> {
    let timestep = orbit.period / (config.frames * config.subframes) as f64;
    let mut bodies = orbit.initial_conds.clone();
    let mut history = vec![bodies.iter().map(|body| body.position).collect()];

    for _ in 0..config.frames {
        for _ in 0..config.subframes {
            step(timestep, &mut bodies);
            history.push(bodies.iter().map(|body| body.position).collect());
        }
    }

    history
}

#[derive(Clone, Debug)]
pub struct SimulationConfig {
    pub frames: usize,
    pub subframes: usize,
}

#[derive(Clone, Debug)]
pub struct Orbit {
    pub initial_conds: Vec<Body>,
    pub period: f64,
}

#[derive(Clone, Debug)]
pub struct Body {
    pub mass: f64,
    pub color: SolidSource,
    pub position: DVec2,
    pub velocity: DVec2,
}

impl Body {
    pub fn apply_force(&mut self, dt: f64, force: DVec2) {
        self.velocity += dt * force * self.mass;
    }
}

pub fn draw(dt: &mut DrawTarget, bodies: &[Body], positions: &[DVec2], a: u8) {
    for (body, position) in bodies.iter().zip(positions.iter()) {
        let mut pb = PathBuilder::new();
        pb.arc(position.x as f32, position.y as f32, 0.08, 0.0, TAU);

        let color =
            SolidSource::from_unpremultiplied_argb(a, body.color.r, body.color.g, body.color.b);

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
            body.apply_force(dt, force * -delta);
            other.apply_force(dt, force * delta);
        }
    }
}

pub fn update(dt: f64, bodies: &mut [Body]) {
    for body in bodies.iter_mut() {
        body.position += body.velocity * dt;
    }
}
