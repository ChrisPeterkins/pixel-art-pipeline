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
    /// Pipeline options.
    pub options: PipelineOptions,
}

/// Options controlling pipeline behavior.
pub struct PipelineOptions {
    /// If true, do not write any files to disk.
    pub dry_run: bool,
    /// If set, only run this phase.
    pub only: Option<String>,
}

impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            only: None,
        }
    }
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
            options: PipelineOptions::default(),
        }
    }

    pub fn with_options(mut self, options: PipelineOptions) -> Self {
        self.options = options;
        self
    }
}

/// Run the full pipeline with all phases.
pub fn run_pipeline(config: Config, base_dir: PathBuf) -> Result<PipelineContext> {
    run_pipeline_with_options(config, base_dir, PipelineOptions::default())
}

/// Run the pipeline with custom options (dry-run, phase filtering).
pub fn run_pipeline_with_options(
    config: Config,
    base_dir: PathBuf,
    options: PipelineOptions,
) -> Result<PipelineContext> {
    use crate::animation::AnimationPhase;
    use crate::output::OutputPhase;
    use crate::packer::PackPhase;
    use crate::palette::PalettePhase;
    use crate::scale::ScalePhase;

    let ctx = PipelineContext::new(config, base_dir).with_options(options);
    let mut ctx = ctx;

    let phases: Vec<Box<dyn PipelinePhase>> = vec![
        Box::new(PalettePhase),
        Box::new(PackPhase),
        Box::new(AnimationPhase),
        Box::new(ScalePhase),
        Box::new(OutputPhase),
    ];

    for phase in &phases {
        // Phase filtering: skip phases that don't match --only
        if let Some(ref only) = ctx.options.only {
            let phase_name = phase.name();
            let matches = phase_name.contains(only.as_str())
                || match only.as_str() {
                    "palettes" | "palette" => phase_name == "palette-management",
                    "sheets" | "packing" => phase_name == "sprite-sheet-packing",
                    "animations" | "animation" => phase_name == "animation-assembly",
                    "scaling" | "scale" => phase_name == "multi-resolution-scaling",
                    "output" => phase_name == "output-serialization",
                    _ => false,
                };
            if !matches {
                log::debug!("Skipping phase '{}' (--only {})", phase_name, only);
                continue;
            }
        }

        log::info!("Running phase: {}", phase.name());
        phase.execute(&mut ctx)?;
    }

    Ok(ctx)
}

/// Build summary for reporting.
pub struct BuildSummary {
    pub sheets_packed: usize,
    pub scaled_variants: usize,
    pub animations_assembled: usize,
    pub files_written: usize,
}

impl BuildSummary {
    pub fn from_context(ctx: &PipelineContext) -> Self {
        let files_written = if ctx.options.dry_run {
            0
        } else {
            let mut count = 0;
            // Each scaled sheet produces a PNG + metadata files
            if !ctx.scaled_sheets.is_empty() {
                for sheet_config in &ctx.config.sheets {
                    let format_count = sheet_config.output_formats.len();
                    let scale_count = ctx
                        .scaled_sheets
                        .iter()
                        .filter(|s| s.sheet_name == sheet_config.name)
                        .count();
                    count += scale_count * (1 + format_count); // PNG + metadata per format
                }
            } else {
                for sheet_config in &ctx.config.sheets {
                    if ctx.sheets.contains_key(&sheet_config.name) {
                        count += 1 + sheet_config.output_formats.len(); // PNG + metadata
                    }
                }
            }
            // Animation outputs
            for anim in &ctx.config.animations {
                for output in &anim.outputs {
                    count += 1; // GIF or strip PNG
                    if output.metadata.unwrap_or(false) {
                        count += 1; // metadata JSON
                    }
                }
            }
            count
        };

        Self {
            sheets_packed: ctx.sheets.len(),
            scaled_variants: ctx.scaled_sheets.len(),
            animations_assembled: ctx.animations.len(),
            files_written,
        }
    }
}
