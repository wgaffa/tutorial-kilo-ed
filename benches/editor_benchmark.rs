use std::io::{Write, Result};
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use kilo_edit::Editor;

struct NullWriter;

impl Write for NullWriter {
    fn write(&mut self, _: &[u8]) -> Result<usize> {
        Ok(1)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

pub fn status_bar_benchmark(c: &mut Criterion) {
    let contents = include_str!("lorem.txt");

    let mut group = c.benchmark_group("Editor::draw_status_bar");
    for size in [2, 32, 64, 128, 254, 1024].iter() {
        let mut writer = NullWriter;

        let mut editor = Editor::new(*size, *size);
        editor.from_str(contents);

        group.bench_function(BenchmarkId::new("draw_status_bar", size), |b| {
            b.iter(|| editor.draw_status_bar(&mut writer));
        });
    }
}

criterion_group!(benches, status_bar_benchmark);
criterion_main!(benches);
