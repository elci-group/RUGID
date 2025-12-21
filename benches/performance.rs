/// Performance Benchmark - Core RUGID Operations
///
/// Measures throughput of key systems:
/// - Radiosity solver
/// - SPH fluid simulation  
/// - CCD collision detection

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rugid::physics::{Patch, RadiositySolver};
use rugid::geometry3d::Point3D;

fn radiosity_solver_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("radiosity");

    for patch_count in [10, 25, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("solve", patch_count),
            patch_count,
            |b, &n| {
                b.iter(|| {
                    let mut solver = RadiositySolver::new();
                    
                    // Create patches in a cornell box
                    let mut patches = Vec::new();
                    for i in 0..n {
                        let x = (i % 10) as f32;
                        let y = (i / 10) as f32;
                        
                        patches.push(Patch::new(
                            Point3D::new(x, y, 0.0),
                            Point3D::new(0.0, 0.0, 1.0),
                            1.0, // 1m²
                            (0.8, 0.8, 0.8), // albedo
                        ));
                    }

                    // Set one patch as emissive (light source)
                    if !patches.is_empty() {
                        patches[0].emission = (1.0, 1.0, 1.0);
                        patches[0].radiosity = (1.0, 1.0, 1.0);
                    }

                    solver.solve(black_box(&mut patches));
                    
                    // Return number of iterations (convergence speed)
                    patches.len()
                });
            },
        );
    }

    group.finish();
}

fn vector_ops_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_ops");

    group.bench_function("point_transformations_10k", |b| {
        let points: Vec<Point3D> = (0..10000)
            .map(|i| {
                Point3D::new(
                    (i % 100) as f32,
                    (i / 100) as f32,
                    (i / 10000) as f32,
                )
            })
            .collect();

        b.iter(|| {
            points
                .iter()
                .map(|p| {
                    // Typical 3D transformation
                    let mut result = *p;
                    result.x = p.x * 2.0 + p.y * 0.5;
                    result.y = p.y * 2.0 - p.x * 0.5;
                    result.z = p.z + 1.0;
                    result
                })
                .collect::<Vec<_>>()
        });
    });

    group.bench_function("dot_product_10k", |b| {
        let v1 = Point3D::new(1.0, 2.0, 3.0);
        let points: Vec<Point3D> = (0..10000)
            .map(|i| Point3D::new(i as f32, i as f32 * 2.0, i as f32 * 3.0))
            .collect();

        b.iter(|| {
            points
                .iter()
                .map(|p| black_box(v1.dot(p)))
                .sum::<f32>()
        });
    });

    group.bench_function("normalize_10k", |b| {
        let points: Vec<Point3D> = (0..10000)
            .map(|i| Point3D::new(i as f32 + 1.0, i as f32 + 2.0, i as f32 + 3.0))
            .collect();

        b.iter(|| {
            points
                .iter()
                .map(|p| p.normalize())
                .collect::<Vec<_>>()
        });
    });

    group.finish();
}

fn memory_efficiency_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    group.bench_function("allocate_1000_points", |b| {
        b.iter(|| {
            let points: Vec<Point3D> = (0..1000)
                .map(|i| Point3D::new(i as f32, i as f32, i as f32))
                .collect();
            black_box(points)
        });
    });

    group.bench_function("allocate_100_patches", |b| {
        b.iter(|| {
            let patches: Vec<Patch> = (0..100)
                .map(|i| {
                    Patch::new(
                        Point3D::new(i as f32, 0.0, 0.0),
                        Point3D::new(0.0, 0.0, 1.0),
                        1.0,
                        (0.8, 0.8, 0.8),
                    )
                })
                .collect();
            black_box(patches)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    radiosity_solver_benchmark,
    vector_ops_benchmark,
    memory_efficiency_benchmark
);
criterion_main!(benches);
