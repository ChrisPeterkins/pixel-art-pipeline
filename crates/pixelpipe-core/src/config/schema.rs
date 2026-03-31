use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub project: ProjectConfig,
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub sheets: Vec<SheetConfig>,
    #[serde(default)]
    pub palettes: PalettesConfig,
    #[serde(default)]
    pub scaling: ScalingConfig,
    #[serde(default)]
    pub animations: Vec<AnimationConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
    #[serde(default = "default_input_dir")]
    pub input_dir: PathBuf,
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("./dist/assets")
}

fn default_input_dir() -> PathBuf {
    PathBuf::from("./raw")
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
    #[serde(default = "default_scale_factors")]
    pub scale_factors: Vec<u32>,
    #[serde(default = "default_padding")]
    pub padding: u32,
    #[serde(default = "default_true")]
    pub power_of_two: bool,
    #[serde(default = "default_max_sheet_size")]
    pub max_sheet_size: u32,
    #[serde(default)]
    pub trim_transparent: bool,
    #[serde(default = "default_true")]
    pub pixel_perfect: bool,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            scale_factors: default_scale_factors(),
            padding: default_padding(),
            power_of_two: true,
            max_sheet_size: default_max_sheet_size(),
            trim_transparent: false,
            pixel_perfect: true,
        }
    }
}

fn default_scale_factors() -> Vec<u32> {
    vec![1]
}

fn default_padding() -> u32 {
    1
}

fn default_max_sheet_size() -> u32 {
    2048
}

fn default_true() -> bool {
    true
}

// ── Sprite Sheet Config ──────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SheetConfig {
    pub name: String,
    pub inputs: Vec<String>,
    #[serde(default = "default_output_formats")]
    pub output_formats: Vec<OutputFormat>,
    pub padding: Option<u32>,
    pub max_sheet_size: Option<u32>,
    pub power_of_two: Option<bool>,
}

fn default_output_formats() -> Vec<OutputFormat> {
    vec![OutputFormat::Phaser]
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Phaser,
    Css,
    Canvas,
}

// ── Palette Config ───────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PalettesConfig {
    #[serde(default)]
    pub definitions: Vec<PaletteDefinition>,
    #[serde(default)]
    pub operations: Vec<PaletteOperation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaletteDefinition {
    pub name: String,
    pub source: Option<String>,
    pub colors: Option<Vec<String>>,
    pub max_colors: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, tag = "type")]
pub enum PaletteOperation {
    #[serde(rename = "enforce")]
    Enforce {
        palette: String,
        targets: Vec<String>,
        #[serde(default = "default_enforce_strategy")]
        strategy: EnforceStrategy,
    },
    #[serde(rename = "swap")]
    Swap {
        source_palette: String,
        target_palette: String,
        inputs: Vec<String>,
        output_dir: Option<PathBuf>,
        output_suffix: Option<String>,
    },
}

fn default_enforce_strategy() -> EnforceStrategy {
    EnforceStrategy::Nearest
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum EnforceStrategy {
    Nearest,
    Error,
    Dither,
}

// ── Scaling Config ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScalingConfig {
    #[serde(default = "default_scale_factors")]
    pub factors: Vec<u32>,
    #[serde(default = "default_naming")]
    pub naming: String,
    #[serde(default = "default_apply_to")]
    pub apply_to: ScaleApplyTo,
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            factors: default_scale_factors(),
            naming: default_naming(),
            apply_to: default_apply_to(),
        }
    }
}

fn default_naming() -> String {
    "{name}@{scale}x".to_string()
}

fn default_apply_to() -> ScaleApplyTo {
    ScaleApplyTo::Sheets
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ScaleApplyTo {
    Sheets,
    Sources,
    Both,
}

// ── Animation Config ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationConfig {
    pub name: String,
    pub frames: Vec<FrameSource>,
    pub timing: TimingConfig,
    pub outputs: Vec<AnimationOutput>,
    pub scale_factors: Option<Vec<u32>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameSource {
    pub pattern: String,
    #[serde(default = "default_sort")]
    pub sort: SortOrder,
}

fn default_sort() -> SortOrder {
    SortOrder::Natural
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Natural,
    Alphabetical,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimingConfig {
    pub frame_duration_ms: Option<u32>,
    pub durations_ms: Option<Vec<u32>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationOutput {
    #[serde(rename = "type")]
    pub output_type: AnimationOutputType,
    #[serde(rename = "loop")]
    pub loop_animation: Option<bool>,
    pub output: Option<String>,
    pub direction: Option<StripDirection>,
    pub metadata: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AnimationOutputType {
    Gif,
    Strip,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StripDirection {
    Horizontal,
    Vertical,
}
