#![recursion_limit = "256"]

#[cfg(feature = "desktop")]
use dioxus::desktop::WindowBuilder;

#[cfg(feature = "desktop")]
use burn::tensor::Tensor;
#[cfg(feature = "desktop")]
use burn::backend::{Autodiff, wgpu::Wgpu};
#[cfg(feature = "desktop")]
use burn::tensor::Distribution;

mod actors;

mod app;

fn main() {
    #[cfg(feature = "desktop")]
    {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "info");
        }
        
        use dioxus::desktop::trayicon::dpi::LogicalSize;
        let window_builder = WindowBuilder::new()
            .with_title("burn-gradient-demo-app - Desktop")
            .with_always_on_top(false)
            .with_inner_size(LogicalSize::new(700.0, 400.0));
        let config = dioxus::desktop::Config::default().with_window(window_builder);
        dioxus::LaunchBuilder::new()
            .with_cfg(config)
            .launch(app::desktop::DesktopApp);
    }
}


#[cfg(feature = "desktop")]
pub fn burn_tensor_example() {
    type Backend = Autodiff<Wgpu>;

    let device = Default::default();

    let x: Tensor<Backend, 2> = Tensor::random([32, 32], Distribution::Default, &device);
    let y: Tensor<Backend, 2> = Tensor::random([32, 32], Distribution::Default, &device).require_grad();

    let tmp = x.clone() + y.clone();
    let tmp = tmp.matmul(x);
    let tmp = tmp.exp();

    let grads = tmp.backward();
    let y_grad = y.grad(&grads).unwrap();
    let grad_msg = format!("[Desktop] Gradient tensor:\n{y_grad}");
    eprintln!("{}", grad_msg);
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/burn-gradient-demo-app-desktop.log") {
        use std::io::Write;
        let _ = writeln!(file, "{}", grad_msg);
        let _ = file.flush();
    }
}
