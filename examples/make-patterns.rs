use std::io::BufWriter;
use std::io::Write;

fn main() {
    let iterations = std::env::args()
        .nth(1)
        .map(|s| s.parse::<usize>().unwrap())
        .unwrap_or(10_000);

    let stdout = std::io::stdout();
    let lock = stdout.lock();
    let mut writer = BufWriter::new(lock);

    for _ in 0..iterations {
        for i in 0..100 {
            for k in ["A", "B", "C"] {
                writeln!(writer, "value config {} {}", k, i).unwrap();
                writeln!(writer, "device {} {}", k, i).unwrap();
            }
        }
    }
}
