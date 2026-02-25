#[cfg(feature = "desktop")]
use dioxus::desktop::WindowBuilder;

// Import Burn neural net building blocks and traits (desktop only)
#[cfg(feature = "desktop")]
use burn::tensor::Tensor;                 // Core tensor (multi-dimensional array) type
#[cfg(feature = "desktop")]
use burn::backend::{Autodiff, wgpu::Wgpu}; // Backend types: Autodiff wrapper and WGPU backend
#[cfg(feature = "desktop")]
use burn::tensor::Distribution;           // Distribution for random tensor generation

// Modules
mod agents;
#[cfg(feature = "desktop")]
mod lstm;

// Platform-specific app modules
mod app;
mod shared;

fn main() {
    // Configure tracing to filter out /api/messages/stream logs
    // This reduces terminal spam from Dioxus server's automatic logging
    #[cfg(feature = "desktop")]
    {
        // Set RUST_LOG to filter out the stream endpoint if not already set
        if std::env::var("RUST_LOG").is_err() {
            // Filter out Dioxus server logs for /api/messages/stream
            // Keep other logs at INFO level
            std::env::set_var("RUST_LOG", "info");
        }
        
        // Window configuration for desktop
        let window_builder = WindowBuilder::new()
            .with_title("pattern-clock - Desktop")
            .with_always_on_top(false);
        let config = dioxus::desktop::Config::default().with_window(window_builder);
        dioxus::LaunchBuilder::new()
            .with_cfg(config)
            .launch(app::desktop::DesktopApp);
    }
    
    #[cfg(all(not(feature = "desktop"), feature = "web"))]
    {
        // Web mode or fullstack server with web client
        // When dx serve runs, it builds server binary with 'web' feature
        // This launches WebApp which will be served to the browser
        dioxus::launch(app::web::WebApp);
    }
    
    #[cfg(all(not(feature = "desktop"), not(feature = "web"), feature = "server"))]
    {
        // Server-only mode - in fullstack, the server should also launch WebApp
        // to serve the web client. This branch should not normally be hit,
        // but if it is, we still launch WebApp to serve the client.
        // The wasm client is built separately and served as static files.
        dioxus::launch(app::web::WebApp);
    }
}


// ============================================================================
// Burn Tensor Example
// ============================================================================

/// Example function demonstrating Burn tensor operations with automatic differentiation
/// 
/// This function demonstrates the core concepts of automatic differentiation (autodiff) in Burn:
/// 1. Creates two random 32x32 tensors (x and y)
/// 2. Marks tensor y to track gradients (require_grad)
/// 3. Performs a series of operations: addition, matrix multiplication, and exponential
/// 4. Computes gradients by calling backward() on the final result
/// 5. Extracts and prints the gradient of y with respect to the final output
/// 
/// The computation graph: tmp = exp((x + y) @ x), where @ is matrix multiplication
/// The gradient shows how much the output changes when y changes, which is essential for training neural networks.
#[cfg(feature = "desktop")]
pub fn burn_tensor_example() {
    // Define the backend type: Autodiff wrapper around WGPU backend for GPU-accelerated computation with gradient tracking
    type Backend = Autodiff<Wgpu>;

    // Create a default device handle (typically the first available GPU or CPU)
    let device = Default::default();

    // Create a random 32x32 tensor x with default distribution (values between 0 and 1)
    let x: Tensor<Backend, 2> = Tensor::random([32, 32], Distribution::Default, &device);
    // Create a random 32x32 tensor y and mark it to track gradients (needed for backpropagation)
    let y: Tensor<Backend, 2> = Tensor::random([32, 32], Distribution::Default, &device).require_grad();

    // Add tensors x and y element-wise, creating a new tensor (clones are needed because tensors are moved)
    let tmp = x.clone() + y.clone();
    // Perform matrix multiplication: multiply the result by tensor x (tmp @ x)
    let tmp = tmp.matmul(x);
    // Apply exponential function element-wise to each value in the tensor (e^value)
    let tmp = tmp.exp();

    // Compute gradients by performing backpropagation through the computation graph
    let grads = tmp.backward();
    // Extract the gradient of tensor y from the computed gradients (how much the output changes w.r.t. y)
    let y_grad = y.grad(&grads).unwrap();
    // Print the gradient tensor to the console and log file
    let grad_msg = format!("[Desktop] Gradient tensor:\n{y_grad}");
    println!("{}", grad_msg);
    eprintln!("{}", grad_msg);
    // Also write to log file
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/pattern-clock-desktop.log") {
        use std::io::Write;
        let _ = writeln!(file, "{}", grad_msg);
        let _ = file.flush();
    }
}
