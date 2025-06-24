use std::{f32::consts::TAU, fs::File};

use glam::{DVec2, dvec2};
use raqote::{DrawOptions, DrawTarget, PathBuilder, SolidSource, Transform};

fn main() {
    let mut bodies = vec![
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
    ];

    let width = 400;
    let height = 400;

    let mut image = File::create("target/orbit.gif").unwrap();
    let mut encoder = gif::Encoder::new(&mut image, width, height, &[]).unwrap();

    encoder.set_repeat(gif::Repeat::Infinite).unwrap();

    let period = 6.325897;

    let frames = 100;

    let subframes = 10usize;
    let subalpha = (255 / subframes) as u8;
    let timestep = period / (frames * subframes) as f64;

    for _ in 0..frames {
        let mut dt = DrawTarget::new(width as _, height as _);
        dt.set_transform(&Transform::scale(100.0, 100.0).then_translate((200.0, 200.0).into()));

        for _ in 0..subframes {
            step(timestep, &mut bodies);
            draw(&mut dt, &bodies, subalpha);
        }

        let mut frame = gif::Frame::from_rgba(width, height, dt.get_data_u8_mut());
        frame.delay = 2;
        frame.dispose = gif::DisposalMethod::Background;
        frame.make_lzw_pre_encoded();

        encoder.write_lzw_pre_encoded_frame(&frame).unwrap();
    }
}

pub struct Orbit {
    pub initial_conds: InitialConditions,
    pub period: f64,
}

pub struct InitialConditions {
    pub bodies: Vec<Body>,
}

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

pub fn draw(dt: &mut DrawTarget, bodies: &[Body], a: u8) {
    for body in bodies.iter() {
        let mut pb = PathBuilder::new();
        pb.arc(
            body.position.x as f32,
            body.position.y as f32,
            0.08,
            0.0,
            TAU,
        );

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
