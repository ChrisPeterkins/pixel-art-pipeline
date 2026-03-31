use crate::config::schema::Config;
use crate::error::Result;
use image::RgbaImage;
use std::collections::HashMap;
use std::path::PathBuf;

/// Holds all state passed between pipeline phases.
pub struct PipelineContext {
    /// The parsed config.
    pub config: Config,
    /// Base directory for resolving input paths.
    pub base_dir: PathBuf,
    /// Loaded images keyed by their source path.
    pub images: HashMap<PathBuf, RgbaImage>,
    /// Packed sprite sheet images keyed by sheet name (1x base).
    pub sheets: HashMap<String, SheetResult>,
    /// Scaled variants of sheets (includes all scale factors).
    pub scaled_sheets: Vec<ScaledSheet>,
    /// Animation results keyed by animation name.
    pub animations: HashMap<String, AnimationResult>,
}

/// Result of packing a sprite sheet.
pub struct SheetResult {
    pub image: RgbaImage,
    pub frames: Vec<FramePlacement>,
    pub width: u32,
    pub height: u32,
}

/// A single sprite's placement in a sheet.
pub struct FramePlacement {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// A scaled variant of a sprite sheet.
pub struct ScaledSheet {
    /// Output name (e.g. "sprites@2x")
    pub name: String,
    /// Original sheet name
    pub sheet_name: String,
    /// Scale factor
    pub scale_factor: u32,
    pub image: RgbaImage,
    pub frames: Vec<FramePlacement>,
    pub width: u32,
    pub height: u32,
}

/// Result of assembling an animation.
pub struct AnimationResult {
    pub strip_image: Option<RgbaImage>,
    pub gif_data: Option<Vec<u8>>,
    pub frame_count: u32,
    pub frame_width: u32,
    pub frame_height: u32,
    pub timing: Vec<u32>,
}

/// Trait for each pipeline phase.
pub trait PipelinePhase {
    fn name(&self) -> &str;
    fn execute(&self, ctx: &mut PipelineContext) -> Result<()>;
}

impl PipelineContext {
    pub fn new(config: Config, base_dir: PathBuf) -> Self {
        Self {
            config,
            base_dir,
            images: HashMap::new(),
            sheets: HashMap::new(),
            scaled_sheets: Vec::new(),
            animations: HashMap::new(),
        }
    }
}

/// Run the full pipeline with all phases.
pub fn run_pipeline(config: Config, base_dir: PathBuf) -> Result<PipelineContext> {
    use crate::output::OutputPhase;
    use crate::packer::PackPhase;
    use crate::palette::PalettePhase;
    use crate::scale::ScalePhase;

    let mut ctx = PipelineContext::new(config, base_dir);

    let phases: Vec<Box<dyn PipelinePhase>> = vec![
        Box::new(PalettePhase),
        Box::new(PackPhase),
        // Animation assembly (Milestone 5)
        Box::new(ScalePhase),
        Box::new(OutputPhase),
    ];

    for phase in &phases {
        log::info!("Running phase: {}", phase.name());
        phase.execute(&mut ctx)?;
    }

    Ok(ctx)
}
