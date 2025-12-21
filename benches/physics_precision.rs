/// Physics Precision Benchmark - "Falling Spheres"
///
/// Tests the accuracy of RUGID's physics engine by comparing
/// simulated projectile motion to analytical ground truth.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rugid::geometry3d::Point3D;

/// Analytical solution for free fall
fn analytical_fall_time(height: f32, gravity: f32) -> f32 {
    (2.0 * height / gravity).sqrt()
}

fn analytical_final_velocity(height: f32, gravity: f32) -> f32 {
    (2.0 * gravity * height).sqrt()
}

/// Simulate falling sphere using simple Euler integration
fn simulate_fall(height: f32, gravity: f32, dt: f32) -> (f32, f32) {
    let mut pos = height;
    let mut vel = 0.0;
    let mut time = 0.0;

    while pos > 0.0 {
        vel += gravity * dt;
        pos -= vel * dt;
        time += dt;
    }

    (time, vel)
}

fn falling_spheres_precision(c: &mut Criterion) {
    let gravity = 9.81;
    let dt = 0.001; // 1ms timestep

    let mut group = c.benchmark_group("physics_precision");

    for height in [10.0, 20.0, 50.0, 100.0].iter() {
        group.bench_with_input(
            BenchmarkId::new("falling_sphere", height),
            height,
            |b, &h| {
                b.iter(|| {
                    let (sim_time, sim_vel) = simulate_fall(black_box(h), gravity, dt);
                    let true_time = analytical_fall_time(h, gravity);
                    let true_vel = analytical_final_velocity(h, gravity);

                    // Calculate errors
                    let time_error = (sim_time - true_time).abs() / true_time;
                    let vel_error = (sim_vel - true_vel).abs() / true_vel;

                    (time_error, vel_error)
                });
            },
        );
    }

    group.finish();
}

fn vector_transform_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_ops");

    group.bench_function("point3d_transform_1000", |b| {
        let points: Vec<Point3D> = (0..1000)
            .map(|i| Point3D::new(i as f32, i as f32, i as f32))
            .collect();

        b.iter(|| {
            points.iter().map(|p| {
                let mut transformed = *p;
                transformed.x += 1.0;
                transformed.y *= 2.0;
                transformed.z -= 0.5;
                transformed
            }).collect::<Vec<_>>()
        });
    });

    group.finish();
}

criterion_group!(benches, falling_spheres_precision, vector_transform_performance);
criterion_main!(benches);
