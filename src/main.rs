use tokio;

use square_homage::run;

#[tokio::main(flavor="current_thread")]
async fn main() {
    // Define my squares
    // Pass the squares into the pipeline???
    // Get a render pass
    // Push the next frame of squares into it??
    // Call the shader somewhere??
    run().await;
}
