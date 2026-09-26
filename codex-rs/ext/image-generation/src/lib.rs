pub mod artifact;
pub mod backend;
pub mod extension;
pub mod tool;

pub use extension::install;

pub(crate) const IMAGE_GEN_NAMESPACE: &str = "image_gen";
pub(crate) const IMAGEGEN_TOOL_NAME: &str = "imagegen";
