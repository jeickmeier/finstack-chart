# Interpolation finite component benchmark

Date: 9 September 2026. Optimized Rust 1.97.1, Apple M5 Max, active desktop with concurrent work. See [environment](environment.json) and [raw batches and allocation accounting](benchmark.json). This is an ADR-008 finite component profile, not a PERF-01–05 release pass.

Each workload has ten warm-up batches and thirty timed batches; nearest-rank p50/p95 are in nanoseconds per operation. Preparation includes cloning/owning the descriptor, bounded wire validation and compilation; endpoint generation occurs before measurement. The first batch is recorded separately as `cold_batch_ns`, not a process-startup measurement. A separate warm batch counts System alloc/alloc_zeroed/realloc calls and requested bytes. The instrumentation shim remains installed during timing with counting disabled. These counts are allocation traffic, not live memory, RSS or a memory-plateau claim.

| Workload | p50 ns/op | p95 ns/op | Allocations/op |
| --- | ---: | ---: | ---: |
| `number/prepare` | 687.1 | 813.8 | 7.00 |
| `number/sample_owned` | 3.8 | 4.2 | 0.00 |
| `number/sample_into` | 5.6 | 5.8 | 0.00 |
| `lab/prepare` | 961.0 | 1598.3 | 13.00 |
| `lab/sample_owned` | 97.1 | 155.2 | 1.00 |
| `lab/sample_into` | 99.0 | 156.5 | 1.00 |
| `lab/sample_color` | 5.0 | 5.2 | 0.00 |
| `transform/prepare` | 916.0 | 1377.7 | 23.00 |
| `transform/sample_owned` | 586.9 | 1043.1 | 15.75 |
| `transform/sample_into` | 576.0 | 682.7 | 15.75 |
| `transform/sample_matrix` | 22.3 | 120.0 | 0.00 |
| `zoom/prepare` | 568.3 | 685.6 | 6.00 |
| `zoom/sample_owned` | 33.5 | 33.8 | 1.00 |
| `zoom/sample_into` | 31.0 | 32.1 | 1.00 |
| `nested-16/prepare` | 33886.9 | 35935.4 | 523.00 |
| `nested-16/sample_owned` | 2109.2 | 2733.1 | 52.57 |
| `nested-16/sample_into` | 1101.2 | 1435.0 | 16.00 |
| `nested-1024/prepare` | 1961014.6 | 2122658.3 | 31795.00 |
| `nested-1024/sample_owned` | 147731.2 | 166506.2 | 3388.90 |
| `nested-1024/sample_into` | 72439.6 | 85016.6 | 1024.00 |
| `registered-number/prepare` | 2250.2 | 3845.6 | 40.00 |
| `registered-number/sample_owned` | 9.6 | 10.2 | 0.00 |
| `registered-number/sample_into` | 11.9 | 12.3 | 0.00 |

Numeric and registered numeric sampling, floating Lab sampling and typed transform matrices allocate zero in the measured warm batch. Text color/transform results and owned zoom/structured results allocate. `sample_into` retains nested container capacity but still allocates formatted strings; it is not advertised as allocation-free. No performance budget or numerical tolerance was weakened, and no optimization gate is inferred from this snapshot.
