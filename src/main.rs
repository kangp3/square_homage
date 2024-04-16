use tokio::runtime::Runtime;
use tokio::task;
use wgpu_bootstrap::run;

fn main() {
    let rt = Runtime::new().unwrap();
    let handle = rt.handle();
    handle.block_on(run());
}
