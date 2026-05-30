use std::time::Instant;

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub read_speed_mbps: f64,
    pub write_speed_mbps: f64,
    pub iops: f64,
    pub duration_secs: f64,
}

pub struct Benchmark;

impl Benchmark {
    pub fn run() -> BenchmarkResult {
        let start = Instant::now();

        let read_speed = Self::measure_read_speed();
        let write_speed = Self::measure_write_speed();
        let iops = Self::measure_iops();

        BenchmarkResult {
            read_speed_mbps: read_speed,
            write_speed_mbps: write_speed,
            iops,
            duration_secs: start.elapsed().as_secs_f64(),
        }
    }

    fn measure_read_speed() -> f64 {
        let tmp = std::env::temp_dir().join("windebloat_bench");
        let test_file = tmp.join("bench_read_test");
        let _ = std::fs::create_dir_all(&tmp);

        let data = vec![0u8; 64 * 1024 * 1024];
        let _ = std::fs::write(&test_file, &data);

        let start = Instant::now();
        let mut total_read = 0u64;
        let buf = &mut [0u8; 65536];
        for _ in 0..1024 {
            if let Ok(mut f) = std::fs::File::open(&test_file) {
                use std::io::Read;
                if let Ok(n) = f.read(buf) {
                    total_read += n as u64;
                }
            }
        }
        let elapsed = start.elapsed().as_secs_f64().max(0.001);
        let _ = std::fs::remove_file(&test_file);

        (total_read as f64 / elapsed) / (1024.0 * 1024.0)
    }

    fn measure_write_speed() -> f64 {
        let tmp = std::env::temp_dir().join("windebloat_bench");
        let test_file = tmp.join("bench_write_test");
        let _ = std::fs::create_dir_all(&tmp);

        let data = vec![0xABu8; 16 * 1024 * 1024];
        let start = Instant::now();

        let mut total_written = 0u64;
        for _ in 0..64 {
            if std::fs::write(&test_file, &data).is_ok() {
                total_written += data.len() as u64;
            }
        }
        let elapsed = start.elapsed().as_secs_f64().max(0.001);
        let _ = std::fs::remove_file(&test_file);
        let _ = std::fs::remove_dir(&tmp);

        (total_written as f64 / elapsed) / (1024.0 * 1024.0)
    }

    fn measure_iops() -> f64 {
        let tmp = std::env::temp_dir().join("windebloat_bench");
        let _ = std::fs::create_dir_all(&tmp);
        let start = Instant::now();
        let mut ops = 0u64;

        for i in 0..1000 {
            let f = tmp.join(format!("iops_{}", i));
            if std::fs::write(&f, &[0u8; 512]).is_ok() {
                ops += 1;
            }
            if std::fs::remove_file(&f).is_ok() {
                ops += 1;
            }
        }
        let elapsed = start.elapsed().as_secs_f64().max(0.001);
        ops as f64 / elapsed
    }

    pub fn format_readable(&self, result: &BenchmarkResult) -> String {
        format!(
            "Okuma: {:.0} MB/s | Yazma: {:.0} MB/s | IOPS: {:.0}",
            result.read_speed_mbps, result.write_speed_mbps, result.iops
        )
    }
}
