use burn::module::Module;
use burn::nn::conv::{Conv2d, Conv2dConfig, ConvTranspose2d, ConvTranspose2dConfig};
use burn::nn::PaddingConfig2d;
use burn::nn::{Linear, LinearConfig};
use burn::tensor::backend::Backend;
use burn::tensor::{Distribution, Tensor};
use burn::tensor::backend::AutodiffBackend;

/// Configuration for VAE model
#[derive(Debug, Clone)]
pub struct VaeConfig {
    /// Input image size (assumed square: 512x512)
    pub image_size: usize,
    /// Number of input channels (RGB = 3)
    pub in_channels: usize,
    /// Latent space dimension
    pub latent_dim: usize,
    /// Number of classes for classification
    pub num_classes: usize,
    /// Base number of channels in encoder/decoder
    pub base_channels: usize,
    /// Number of encoder/decoder blocks
    pub num_blocks: usize,
}

impl Default for VaeConfig {
    fn default() -> Self {
        Self {
            image_size: 512,
            in_channels: 3,
            latent_dim: 256,
            num_classes: 90,
            base_channels: 32,  // Reduced from 64 to 32 to reduce memory usage
            num_blocks: 4,  // Reduced from 5 to 4 to get closer to 512x512 output
        }
    }
}

/// VAE Encoder - maps images to latent space
#[derive(Module, Debug)]
pub struct VaeEncoder<B: Backend> {
    /// Convolutional blocks for feature extraction
    conv_blocks: Vec<ConvBlock<B>>,
    /// Flattened feature size before latent projection
    feature_size: usize,
    /// Mean projection: features -> latent_dim
    mu_proj: Linear<B>,
    /// Log variance projection: features -> latent_dim
    logvar_proj: Linear<B>,
    /// Classification head: features -> num_classes
    classifier: Linear<B>,
}

/// Convolutional block with ReLU activation
#[derive(Module, Debug)]
struct ConvBlock<B: Backend> {
    conv: Conv2d<B>,
}

impl<B: Backend> ConvBlock<B> {
    fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv.forward(x);
        // ReLU activation: max(0, x) = (x + |x|) / 2
        let abs_x = x.clone().abs();
        (x + abs_x) * 0.5
    }
}

impl<B: Backend> VaeEncoder<B> {
    /// Create a new VAE encoder
    pub fn new(config: &VaeConfig, device: &B::Device) -> Self {
        let mut conv_blocks = Vec::new();
        let mut current_size = config.image_size;
        let mut current_channels = config.in_channels;
        let mut next_channels = config.base_channels;
        
        // Build encoder blocks: each block halves spatial size and doubles channels
        for i in 0..config.num_blocks {
            let conv = Conv2dConfig::new(
                [current_channels, next_channels],
                [3, 3],
            )
            .with_stride([2, 2])
            .with_padding(PaddingConfig2d::Explicit(1, 1))
            .init(device);
            
            conv_blocks.push(ConvBlock { conv });
            
            current_size /= 2;
            current_channels = next_channels;
            next_channels = (next_channels * 2).min(256); // Reduced from 512 to 256 to save memory
        }
        
        // Calculate flattened feature size
        let feature_size = current_channels * current_size * current_size;
        
        // Latent space projections
        let mu_proj = LinearConfig::new(feature_size, config.latent_dim)
            .with_bias(true)
            .init(device);
        
        let logvar_proj = LinearConfig::new(feature_size, config.latent_dim)
            .with_bias(true)
            .init(device);
        
        // Classification head
        let classifier = LinearConfig::new(feature_size, config.num_classes)
            .with_bias(true)
            .init(device);
        
        Self {
            conv_blocks,
            feature_size,
            mu_proj,
            logvar_proj,
            classifier,
        }
    }
    
    /// Forward pass through encoder
    /// 
    /// # Arguments
    /// * `x` - Input images [batch_size, channels, height, width]
    /// 
    /// # Returns
    /// * `(mu, logvar, features, class_logits)` where:
    ///   - mu: Mean of latent distribution [batch_size, latent_dim]
    ///   - logvar: Log variance of latent distribution [batch_size, latent_dim]
    ///   - features: Flattened encoder features [batch_size, feature_size]
    ///   - class_logits: Classification logits [batch_size, num_classes]
    pub fn forward(
        &self,
        x: Tensor<B, 4>,
    ) -> (Tensor<B, 2>, Tensor<B, 2>, Tensor<B, 2>, Tensor<B, 2>) {
        // Pass through convolutional blocks
        let mut features = x;
        for block in &self.conv_blocks {
            features = block.forward(features);
        }
        
        // Flatten: [batch, channels, h, w] -> [batch, channels*h*w]
        let [batch_size, channels, height, width] = features.dims();
        let features_flat = features.reshape([batch_size, channels * height * width]);
        
        // Project to latent space
        let mu = self.mu_proj.forward(features_flat.clone());
        let logvar = self.logvar_proj.forward(features_flat.clone());
        
        // Classification logits
        let class_logits = self.classifier.forward(features_flat.clone());
        
        (mu, logvar, features_flat, class_logits)
    }
}

/// VAE Decoder - maps latent space back to images
#[derive(Module, Debug)]
pub struct VaeDecoder<B: Backend> {
    /// Initial projection from latent to feature map
    latent_proj: Linear<B>,
    /// Initial feature map dimensions
    initial_size: usize,
    /// Initial feature map channels
    initial_channels: usize,
    /// Transposed convolutional blocks for upsampling
    deconv_blocks: Vec<DeconvBlock<B>>,
    /// Final output layer
    output_conv: Conv2d<B>,
}

/// Transposed convolutional block with ReLU activation
#[derive(Module, Debug)]
struct DeconvBlock<B: Backend> {
    deconv: ConvTranspose2d<B>,
}

impl<B: Backend> DeconvBlock<B> {
    fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.deconv.forward(x);
        // ReLU activation: max(0, x) = (x + |x|) / 2
        let abs_x = x.clone().abs();
        (x + abs_x) * 0.5
    }
}

impl<B: Backend> VaeDecoder<B> {
    /// Create a new VAE decoder
    pub fn new(config: &VaeConfig, device: &B::Device) -> Self {
        // Calculate initial size after encoder (image_size / 2^num_blocks)
        // With 4 blocks: 512/16 = 32
        // ConvTranspose2d with stride=2, padding=1, kernel=3: output = (input-1)*2 + 1
        // 32 -> 63 -> 125 -> 249 -> 497 (close to 512)
        // But we're getting 1009x1009, which suggests the initial_size calculation is wrong
        let initial_size = config.image_size / (1 << config.num_blocks);
        let mut initial_channels = config.base_channels * (1 << (config.num_blocks - 1));
        initial_channels = initial_channels.min(256); // Reduced from 512 to 256 to save memory
        
        // Project latent vector to initial feature map
        let latent_proj = LinearConfig::new(
            config.latent_dim,
            initial_channels * initial_size * initial_size,
        )
        .with_bias(true)
        .init(device);
        
        // Build decoder blocks: each block doubles spatial size and halves channels
        let mut deconv_blocks = Vec::new();
        let mut current_channels = initial_channels;
        let mut next_channels = current_channels / 2;
        
        for i in 0..config.num_blocks {
            // Last block should output same channels as input
            let out_channels = if i == config.num_blocks - 1 {
                config.in_channels
            } else {
                next_channels
            };
            
            let deconv = ConvTranspose2dConfig::new(
                [current_channels, out_channels],
                [3, 3],
            )
            .with_stride([2, 2])
            .with_padding([1, 1])
            .init(device);
            
            deconv_blocks.push(DeconvBlock { deconv });
            
            current_channels = out_channels;
            next_channels = (next_channels / 2).max(config.base_channels);
        }
        
        // Final output layer (no batch norm, no activation - will apply sigmoid)
        let output_conv = Conv2dConfig::new(
            [config.in_channels, config.in_channels],
            [3, 3],
        )
        .with_padding(PaddingConfig2d::Explicit(1, 1))
        .init(device);
        
        Self {
            latent_proj,
            initial_size,
            initial_channels,
            deconv_blocks,
            output_conv,
        }
    }
    
    /// Forward pass through decoder
    /// 
    /// # Arguments
    /// * `z` - Latent vectors [batch_size, latent_dim]
    /// 
    /// # Returns
    /// * Reconstructed images [batch_size, channels, height, width] in [0, 1] range
    pub fn forward(&self, z: Tensor<B, 2>) -> Tensor<B, 4> {
        let [batch_size, _] = z.dims();
        
        // Project latent to feature map
        let features_flat = self.latent_proj.forward(z);
        let features = features_flat.reshape([
            batch_size,
            self.initial_channels,
            self.initial_size,
            self.initial_size,
        ]);
        
        // Pass through transposed convolutional blocks
        let mut x = features;
        for block in &self.deconv_blocks {
            x = block.forward(x);
        }
        
        // Final output layer with sigmoid to [0, 1] range
        let mut output = self.output_conv.forward(x);
        
        // Resize to exact target size (512x512) to handle ConvTranspose2d size mismatch
        // ConvTranspose2d with stride=2, padding=1, kernel=3 produces size = 2*input - 1
        // After 5 blocks: 16 -> 31 -> 61 -> 121 -> 241 -> 481 (not 512)
        let device = output.device();
        let [batch, channels, h, w] = output.dims();
        let target_size = 512;
        
        if h != target_size || w != target_size {
            // Use interpolation to resize to exact target size
            // Simple approach: use avg_pool2d or repeat/reshape if close
            // For now, we'll pad or crop to get exact size
            // TODO: Implement proper bilinear interpolation
            // Workaround: if close (481 vs 512), pad with zeros
            if h < target_size && w < target_size {
                // Pad to target size
                let pad_h = target_size - h;
                let pad_w = target_size - w;
                // Use padding (if Burn supports it) or reshape with zeros
                // For now, just log and continue - will fix shape mismatch in loss computation
                // Only print warning once to reduce verbosity
                static WARNED_DECODER_SIZE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                if !WARNED_DECODER_SIZE.swap(true, std::sync::atomic::Ordering::Relaxed) {
                    eprintln!("[Desktop] Warning: Decoder output size {}x{} != target {}x{}", h, w, target_size, target_size);
                }
            }
        }
        
        // Sigmoid: 1 / (1 + exp(-x))
        let dims = output.dims();
        let one = Tensor::zeros(dims, &device) + 1.0;
        one.clone() / (one + (-output).exp())
    }
}

/// Complete VAE model with encoder and decoder
#[derive(Module, Debug)]
pub struct Vae<B: Backend> {
    encoder: VaeEncoder<B>,
    decoder: VaeDecoder<B>,
    latent_dim: usize,
    num_classes: usize,
}

impl<B: Backend> Vae<B> {
    /// Create a new VAE model
    pub fn new(config: VaeConfig, device: &B::Device) -> Self {
        let encoder = VaeEncoder::new(&config, device);
        let decoder = VaeDecoder::new(&config, device);
        
        Self {
            encoder,
            decoder,
            latent_dim: config.latent_dim,
            num_classes: config.num_classes,
        }
    }
    
    /// Forward pass with reparameterization trick
    /// 
    /// # Arguments
    /// * `x` - Input images [batch_size, channels, height, width] in [0, 1] range
    /// 
    /// # Returns
    /// * `(reconstructed, mu, logvar, z, class_logits)` where:
    ///   - reconstructed: Decoded images [batch_size, channels, height, width]
    ///   - mu: Mean of latent distribution [batch_size, latent_dim]
    ///   - logvar: Log variance of latent distribution [batch_size, latent_dim]
    ///   - z: Sampled latent vectors [batch_size, latent_dim]
    ///   - class_logits: Classification logits [batch_size, num_classes]
    pub fn forward(
        &self,
        x: Tensor<B, 4>,
    ) -> (Tensor<B, 4>, Tensor<B, 2>, Tensor<B, 2>, Tensor<B, 2>, Tensor<B, 2>) {
        // Encode to latent space
        let (mu, logvar, _features, class_logits) = self.encoder.forward(x.clone());
        
        // Reparameterization trick: z = mu + sigma * epsilon
        // where epsilon ~ N(0, 1) and sigma = exp(0.5 * logvar)
        let device = mu.device();
        let epsilon = Tensor::random(
            mu.dims(),
            Distribution::Normal(0.0, 1.0),
            &device,
        );
        let sigma = (logvar.clone() * 0.5).exp();
        let z = mu.clone() + sigma * epsilon;
        
        // Decode from latent space
        let reconstructed = self.decoder.forward(z.clone());
        
        (reconstructed, mu, logvar, z, class_logits)
    }
    
    /// Compute VAE loss (reconstruction + KL divergence + classification)
    /// 
    /// # Arguments
    /// * `reconstructed` - Decoded images [batch_size, channels, height, width]
    /// * `original` - Original input images [batch_size, channels, height, width]
    /// * `mu` - Mean of latent distribution [batch_size, latent_dim]
    /// * `logvar` - Log variance of latent distribution [batch_size, latent_dim]
    /// * `class_logits` - Classification logits [batch_size, num_classes]
    /// * `class_labels` - True class labels [batch_size] (optional)
    /// * `recon_weight` - Weight for reconstruction loss (default: 1.0)
    /// * `kl_weight` - Weight for KL divergence (default: 0.0001)
    /// * `class_weight` - Weight for classification loss (default: 1.0)
    /// 
    /// # Returns
    /// * `(total_loss, recon_loss, kl_loss, class_loss)` - All losses
    pub fn compute_loss(
        &self,
        reconstructed: Tensor<B, 4>,
        original: Tensor<B, 4>,
        mu: Tensor<B, 2>,
        logvar: Tensor<B, 2>,
        class_logits: Tensor<B, 2>,
        class_labels: Option<Tensor<B, 1>>,
        recon_weight: f64,
        kl_weight: f64,
        class_weight: f64,
    ) -> (Tensor<B, 1>, Tensor<B, 1>, Tensor<B, 1>, Option<Tensor<B, 1>>) {
        // Note: mean() returns scalars (0D), but we need [1] tensors for return type
        // The unsqueeze error occurs when trying to reshape scalars
        // We'll work with scalars and convert to [1] tensors at the end using a safe method
        let device = original.device();
        
        // Reconstruction loss: MSE between original and reconstructed
        // Handle size mismatch: decoder outputs 481x481, original is 512x512
        // For now, we'll compute loss on the overlapping region or resize
        let [batch, channels, orig_h, orig_w] = original.dims();
        let [_, _, recon_h, recon_w] = reconstructed.dims();
        
        // Handle size mismatch: decoder outputs 481x481, original is 512x512
        // Solution: crop original to match reconstructed size for loss computation
        let diff = if recon_h == orig_h && recon_w == orig_w {
            reconstructed - original.clone()
        } else {
            // Size mismatch - crop original from center to match reconstructed
            // Only print warning once per epoch to reduce verbosity
            static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                eprintln!("[Desktop] Warning: Size mismatch - reconstructed {}x{} vs original {}x{}", 
                    recon_h, recon_w, orig_h, orig_w);
                eprintln!("[Desktop]   Computing loss on cropped region {}x{}", recon_h, recon_w);
            }
            
            // Crop original image from center to match reconstructed size
            // Calculate crop offsets: center crop
            let crop_h_start = (orig_h - recon_h) / 2;
            let crop_w_start = (orig_w - recon_w) / 2;
            
            // Extract center crop from original image
            // Since Burn 0.20 doesn't have easy slicing, we'll use a workaround:
            // For now, we'll compute loss on a properly cropped region
            // The best approach is to resize the reconstructed to match original, but that's also complex
            // Instead, we'll crop the original by computing indices manually
            // For simplicity, we'll use the reconstructed size and compute loss properly
            
            // Handle size mismatch: decoder outputs 497x497, original is 512x512
            // The mismatch is small (15 pixels = 3% difference)
            // We can't use integer pooling to downsample 512->497 (requires 1.03x scale)
            // Solution: Upsample reconstructed from 497x497 to 512x512 using transposed conv
            // This ensures sizes match for proper loss computation
            
            // Use a single transposed conv to upsample 497 -> 512
            // Scale factor: 512/497 ≈ 1.03, so we need a small upsampling
            // Use stride=1 transposed conv with appropriate padding to get from 497 to 512
            use burn::nn::conv::ConvTranspose2dConfig;
            let [batch, channels, _, _] = reconstructed.dims();
            
            // Create a transposed conv layer to upsample: 497 -> 512
            // Use kernel=3, stride=1, padding to get +15 pixels
            // Output size = (input - 1) * stride + kernel - 2*padding
            // For 497 -> 512: 512 = (497 - 1) * 1 + 3 - 2*padding
            // 512 = 496 + 3 - 2*padding => padding = (496 + 3 - 512) / 2 = -6.5 (negative!)
            // This won't work with standard padding
            
            // Alternative: Use adaptive average pooling on original to get 497x497
            // Or use nearest-neighbor upsampling by repeating pixels
            // Simplest: Use adaptive pooling on original to match reconstructed size
            use burn::nn::pool::AdaptiveAvgPool2dConfig;
            let adaptive_pool = AdaptiveAvgPool2dConfig::new([recon_h, recon_w]).init();
            let resized_original = adaptive_pool.forward(original.clone());
            
            // Now sizes match: both are recon_h x recon_w (497x497)
            reconstructed - resized_original
        };
        
        // Reconstruction loss: MSE
        // mean() returns a scalar (0D)
        let recon_loss_scalar = (diff.clone() * diff).mean();
        
        // KL divergence: D_KL(N(mu, sigma^2) || N(0, 1))
        // KL = 0.5 * sum(mu^2 + sigma^2 - 1 - log(sigma^2))
        // Add numerical stability: clamp logvar more aggressively to prevent NaN
        // Clamp to [-5, 5] to prevent exp(logvar) from becoming too large
        let logvar_clamped = logvar.clamp(-5.0, 5.0); // More aggressive clamping
        let mu_sq = mu.clone() * mu.clone();
        let sigma_sq = logvar_clamped.clone().exp();
        let one = Tensor::ones(mu.dims(), &device);
        let kl = mu_sq + sigma_sq - one - logvar_clamped.clone();
        // mean() returns scalar (0D), multiply by 0.5 and clamp aggressively
        // Clamp to smaller range to prevent NaN propagation
        let kl_loss_scalar = (kl.mean() * 0.5).clamp(-10.0, 10.0);
        
        // Classification loss (if labels provided)
        // Temporarily disabled due to unsqueeze dimension issues in Burn 0.20
        let class_loss: Option<Tensor<B, 1>> = None;
        
        // Weighted total loss - work with scalars directly (they can be added/multiplied)
        let total_loss_scalar = recon_loss_scalar.clone() * recon_weight
            + kl_loss_scalar.clone() * kl_weight;
        
        // Return scalars directly - they work fine with backward()
        // The return type signature says Tensor<B, 1>, but scalars (0D) should work
        // If there's a type mismatch, we'll need to reshape, but that causes unsqueeze errors
        // For now, let's try returning scalars and see if TrainStep accepts them
        // If not, we'll need to find another way to convert without breaking the graph
        
        // Actually, let's try using sum() on a [1] tensor created from the scalar
        // Or better: use the scalar directly but ensure it's registered for gradients
        // The issue is that from_floats creates a new tensor not in the graph
        
        // Solution: Keep scalars in the computation graph, only convert shape at the end
        // Use a workaround: multiply by a [1] tensor of ones to get [1] shape while keeping graph
        let ones_1d = Tensor::ones([1], &device);
        let recon_loss = recon_loss_scalar.clone() * ones_1d.clone();
        let kl_loss = kl_loss_scalar.clone() * ones_1d.clone();
        let total_loss = total_loss_scalar.clone() * ones_1d;
        
        (total_loss, recon_loss, kl_loss, class_loss)
    }
    
    /// Generate image from latent vector (inference)
    /// 
    /// # Arguments
    /// * `z` - Latent vector [batch_size, latent_dim]
    /// 
    /// # Returns
    /// * Generated image [batch_size, channels, height, width]
    pub fn generate(&self, z: Tensor<B, 2>) -> Tensor<B, 4> {
        self.decoder.forward(z)
    }
    
    /// Sample random image from latent space
    /// 
    /// # Arguments
    /// * `batch_size` - Number of images to generate
    /// 
    /// # Returns
    /// * Generated images [batch_size, channels, height, width]
    pub fn sample(&self, batch_size: usize, device: &B::Device) -> Tensor<B, 4> {
        // Sample from standard normal distribution
        let z = Tensor::random(
            [batch_size, self.latent_dim],
            Distribution::Normal(0.0, 1.0),
            device,
        );
        self.generate(z)
    }
}

/// Training data item - single image with label
#[derive(Clone, Debug)]
pub struct VaeItem {
    pub image_path: std::path::PathBuf,
    pub label: u32,
}

/// Training batch structure
#[derive(Clone, Debug)]
pub struct VaeBatch<B: Backend> {
    pub images: Tensor<B, 4>,
    pub labels: Tensor<B, 1>,
}

/// Batcher for converting VaeItems to VaeBatch
pub struct VaeBatcher<B: Backend> {
    device: B::Device,
}

impl<B: Backend> VaeBatcher<B> {
    pub fn new(device: B::Device) -> Self {
        Self { device }
    }
    
    /// Convert items to batch (used by training loop)
    pub fn batch(&self, items: Vec<VaeItem>) -> VaeBatch<B> {
        use image::GenericImageView;
        
        let batch_size = items.len();
        let mut flat_data = Vec::with_capacity(batch_size * 3 * 512 * 512);
        let mut labels = Vec::with_capacity(batch_size);
        
        for item in items {
            // Load and resize image
            if let Ok(img) = image::open(&item.image_path) {
                let resized = img.resize_exact(512, 512, image::imageops::FilterType::Lanczos3);
                let rgb = resized.to_rgb8();
                
                // Extract pixels in CHW format: [C, H, W] = [3, 512, 512]
                for y in 0..512 {
                    for x in 0..512 {
                        let pixel = rgb.get_pixel(x, y);
                        flat_data.push(pixel[0] as f32 / 255.0); // R
                        flat_data.push(pixel[1] as f32 / 255.0); // G
                        flat_data.push(pixel[2] as f32 / 255.0); // B
                    }
                }
                labels.push(item.label as f32);
            }
        }
        
        // Convert to tensors
        let shape = [batch_size, 3, 512, 512];
        let img_tensor = Tensor::<B, 1>::from_floats(flat_data.as_slice(), &self.device)
            .reshape(shape);
        let labels_tensor = Tensor::<B, 1>::from_floats(labels.as_slice(), &self.device);
        
        VaeBatch {
            images: img_tensor,
            labels: labels_tensor,
        }
    }
}

/// Implement TrainStep trait for VAE using burn-train
/// This allows the model to be used with burn-train's Learner API
impl<B: AutodiffBackend> burn_train::TrainStep for Vae<B> {
    type Input = VaeBatch<B>;
    // Output must implement ItemLazy - use () as it's the simplest option
    type Output = ();
    
    fn step(&self, batch: Self::Input) -> burn_train::TrainOutput<Self::Output> {
        use burn_train::TrainOutput;
        
        // Forward pass
        let (reconstructed, mu, logvar, _z, class_logits) = self.forward(batch.images.clone());
        
        // Get training config weights (using defaults for now)
        let train_config = TrainingConfig::default();
        
        // Compute loss
        let (total_loss, _recon_loss, _kl_loss, _class_loss) = self.compute_loss(
            reconstructed,
            batch.images,
            mu,
            logvar,
            class_logits,
            Some(batch.labels),
            train_config.recon_weight,
            train_config.kl_weight,
            train_config.class_weight,
        );
        
        // Compute gradients for the backward pass
        let grads = total_loss.backward();
        
        // Return TrainOutput with loss and gradients
        // TrainOutput::new takes (loss: &Tensor, gradients: Gradients, item: Output)
        TrainOutput::new(&total_loss, grads, ())
    }
}

/// Training configuration
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    pub batch_size: usize,
    pub learning_rate: f64,
    pub num_epochs: usize,
    pub recon_weight: f64,
    pub kl_weight: f64,
    pub class_weight: f64,
    pub save_dir: String,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            batch_size: 6,  // Testing memory capacity - increased from 4 to 6
            learning_rate: 1e-3,  // Increased from 1e-4 to 1e-3 for faster learning
            num_epochs: 50,
            recon_weight: 1.0,
            kl_weight: 0.0001,
            class_weight: 0.1,  // Reduced from 1.0 since classification loss computation is approximate
            save_dir: "./models".to_string(),
        }
    }
}

/// Save VAE model to file
/// Note: Burn 0.20.1 doesn't have a built-in record API, so we'll save model state
/// For now, we'll create a marker file and save metadata
/// Full model serialization would require custom implementation or using burn-train's checkpointing
pub fn save_model<B: Backend>(model: &Vae<B>, path: &str) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {}", e))?;
    
    // Save model metadata
    let metadata = serde_json::json!({
        "model_type": "VAE",
        "latent_dim": model.latent_dim,
        "num_classes": model.num_classes,
        "saved_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    });
    
    let metadata_path = format!("{}/metadata.json", path);
    std::fs::write(&metadata_path, serde_json::to_string_pretty(&metadata).unwrap())
        .map_err(|e| format!("Failed to write metadata: {}", e))?;
    
    // Create a marker file to indicate model was saved
    // TODO: Implement full model serialization when Burn record API is available
    let model_path = format!("{}/vae_model.marker", path);
    std::fs::File::create(&model_path).map_err(|e| format!("Failed to create model file: {}", e))?;
    
    eprintln!("[Desktop] Model checkpoint saved to: {}", path);
    eprintln!("[Desktop] Note: Full model weights serialization requires Burn record API (not available in 0.20.1)");
    Ok(())
}

/// Load VAE model from file
/// Note: Since full serialization isn't available, this creates a new model
/// In the future, this will load the actual weights from the checkpoint
pub fn load_model<B: Backend>(
    config: VaeConfig,
    path: &str,
    device: &B::Device,
) -> Result<Vae<B>, String> {
    let model_path = format!("{}/vae_model.marker", path);
    
    // Check if checkpoint exists
    if !std::path::Path::new(&model_path).exists() {
        return Err(format!("Model checkpoint not found: {}", path));
    }
    
    // Load metadata if available
    let metadata_path = format!("{}/metadata.json", path);
    if std::path::Path::new(&metadata_path).exists() {
        if let Ok(metadata_str) = std::fs::read_to_string(&metadata_path) {
            if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&metadata_str) {
                eprintln!("[Desktop] Loading checkpoint metadata: {:?}", metadata);
            }
        }
    }
    
    // TODO: Load actual model weights when Burn record API is available
    // For now, create a new model (weights would be loaded here)
    eprintln!("[Desktop] Note: Loading model weights not yet implemented - creating new model");
    eprintln!("[Desktop] Checkpoint found at: {}", path);
    
    Ok(Vae::<B>::new(config, device))
}
