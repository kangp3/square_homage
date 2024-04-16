use tokio::runtime::Runtime;
use tokio::task;
use wgpu_bootstrap::run;

fn main() {
    let clear_color = wgpu::Color {
        r: 0.3,
        g: 0.4,
        b: 0.8,
        a: 1.0,
    };

    let rt = Runtime::new().unwrap();
    let handle = rt.handle();
    handle.block_on(run(clear_color));
}
