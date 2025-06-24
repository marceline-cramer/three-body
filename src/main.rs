use std::{f32::consts::TAU, fs::File};

use glam::{DVec2, dvec2};
use raqote::{DrawOptions, DrawTarget, PathBuilder, SolidSource, Transform};
use rayon::{iter::ParallelIterator, slice::ParallelSlice};

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
    render(&config, &orbit, &simulated);
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
