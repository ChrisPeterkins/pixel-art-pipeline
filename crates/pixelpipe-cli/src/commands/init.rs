use pixelpipe_core::error::PipelineError;
use std::path::Path;

const MINIMAL_TEMPLATE: &str = r#"# pixelpipe.yaml — Pixel art asset pipeline config
project:
  name: "my-project"
  input_dir: "./raw"
  output_dir: "./dist/assets"

defaults:
  scale_factors: [1, 2]
  padding: 1
  power_of_two: true

sheets:
  - name: "sprites"
    inputs: ["sprites/*.png"]
    output_formats: ["phaser"]
"#;

const FULL_TEMPLATE: &str = r#"# pixelpipe.yaml — Pixel art asset pipeline config
project:
  name: "my-project"
  input_dir: "./raw"
  output_dir: "./dist/assets"

defaults:
  scale_factors: [1, 2, 4]
  padding: 1
  power_of_two: true
  max_sheet_size: 2048

sheets:
  - name: "ui-elements"
    inputs:
      - "ui/icons/*.png"
      - "ui/buttons/*.png"
    output_formats: ["phaser", "css"]
    padding: 2

  - name: "terrain-tiles"
    inputs:
      - "tiles/terrain/*.png"
    output_formats: ["phaser", "canvas"]

palettes:
  definitions:
    - name: "main-palette"
      source: "reference/master-palette.png"
      max_colors: 32
  operations:
    - type: "enforce"
      palette: "main-palette"
      targets: ["tiles/**/*.png"]
      strategy: "nearest"

scaling:
  factors: [1, 2, 4]
  naming: "{name}@{scale}x"
  apply_to: "sheets"

animations:
  - name: "hero-walk"
    frames:
      - pattern: "characters/hero/walk_*.png"
        sort: "natural"
    timing:
      frame_duration_ms: 100
    outputs:
      - type: "gif"
        loop: true
      - type: "strip"
        direction: "horizontal"
        metadata: true
"#;

pub fn run(template: &str) -> pixelpipe_core::error::Result<()> {
    let config_path = Path::new("pixelpipe.yaml");

    if config_path.exists() {
        return Err(PipelineError::Config(
            "pixelpipe.yaml already exists. Remove it first to reinitialize.".to_string(),
        ));
    }

    let content = match template {
        "minimal" => MINIMAL_TEMPLATE,
        "full" => FULL_TEMPLATE,
        other => {
            return Err(PipelineError::Config(format!(
                "Unknown template '{}'. Available: minimal, full",
                other
            )));
        }
    };

    std::fs::write(config_path, content).map_err(|e| PipelineError::Io {
        path: config_path.to_path_buf(),
        source: e,
    })?;

    println!("Created pixelpipe.yaml (template: {})", template);
    println!("Edit the config and run `pixelpipe validate` to check it.");
    Ok(())
}
