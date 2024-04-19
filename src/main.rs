use tokio;

use square_homage::run;

#[tokio::main(flavor="current_thread")]
async fn main() {
    run().await;
}
