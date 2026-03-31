use pixelpipe_core::pipeline::PipelineContext;
use std::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Atlas,
    Palette,
    Animation,
    Config,
}

// ── Atlas Viewer State ───────────────────────────────────────

pub struct AtlasViewerState {
    pub selected_sheet: Option<String>,
    pub zoom: f32,
    pub selected_frame: Option<usize>,
    pub show_frame_borders: bool,
}

impl Default for AtlasViewerState {
    fn default() -> Self {
        Self {
            selected_sheet: None,
            zoom: 4.0,
            selected_frame: None,
            show_frame_borders: true,
        }
    }
}

// ── Palette Editor State ─────────────────────────────────────

pub struct PaletteEditorState {
    pub palettes: Vec<LoadedPalette>,
    pub selected_palette: Option<usize>,
    pub selected_color: Option<usize>,
    pub edit_color: [f32; 4],
}

impl Default for PaletteEditorState {
    fn default() -> Self {
        Self {
            palettes: Vec::new(),
            selected_palette: None,
            selected_color: None,
            edit_color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

pub struct LoadedPalette {
    pub name: String,
    pub colors: Vec<[u8; 4]>,
}

// ── Config Editor State ──────────────────────────────────────

pub struct ConfigEditorState {
    pub yaml_text: String,
    pub dirty: bool,
    pub validation_error: Option<String>,
    pub validation_ok: bool,
}

impl Default for ConfigEditorState {
    fn default() -> Self {
        Self {
            yaml_text: String::new(),
            dirty: false,
            validation_error: None,
            validation_ok: false,
        }
    }
}

// ── Build Runner State ───────────────────────────────────────

pub struct BuildRunnerState {
    pub status: BuildStatus,
    pub last_error: Option<String>,
}

impl Default for BuildRunnerState {
    fn default() -> Self {
        Self {
            status: BuildStatus::Idle,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuildStatus {
    Idle,
    Running,
    Success { duration_ms: u128 },
    Failed,
}

pub enum BuildMessage {
    Complete(Result<PipelineContext, String>),
    Duration(u128),
}

pub struct BuildChannel {
    pub rx: mpsc::Receiver<BuildMessage>,
}
