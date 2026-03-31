pub mod schema;

use crate::error::{PipelineError, Result};
use schema::Config;
use std::path::Path;

pub fn load_config(path: &Path) -> Result<Config> {
    let contents = std::fs::read_to_string(path).map_err(|e| PipelineError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let config: Config =
        serde_yaml_ng::from_str(&contents).map_err(|e| PipelineError::ConfigParse {
            path: path.to_path_buf(),
            source: e,
        })?;

    validate_config(&config)?;

    Ok(config)
}

fn validate_config(config: &Config) -> Result<()> {
    if config.project.name.is_empty() {
        return Err(PipelineError::Config(
            "project.name cannot be empty".to_string(),
        ));
    }

    for sheet in &config.sheets {
        if sheet.name.is_empty() {
            return Err(PipelineError::Config(
                "Sheet name cannot be empty".to_string(),
            ));
        }
        if sheet.inputs.is_empty() {
            return Err(PipelineError::Config(format!(
                "Sheet '{}' has no inputs",
                sheet.name
            )));
        }
    }

    for anim in &config.animations {
        if anim.name.is_empty() {
            return Err(PipelineError::Config(
                "Animation name cannot be empty".to_string(),
            ));
        }
        if anim.frames.is_empty() {
            return Err(PipelineError::Config(format!(
                "Animation '{}' has no frames",
                anim.name
            )));
        }
        if anim.timing.frame_duration_ms.is_none() && anim.timing.durations_ms.is_none() {
            return Err(PipelineError::Config(format!(
                "Animation '{}' must specify frame_duration_ms or durations_ms",
                anim.name
            )));
        }
    }

    for palette_def in &config.palettes.definitions {
        if palette_def.name.is_empty() {
            return Err(PipelineError::Config(
                "Palette name cannot be empty".to_string(),
            ));
        }
        if palette_def.source.is_none() && palette_def.colors.is_none() {
            return Err(PipelineError::Config(format!(
                "Palette '{}' must specify either 'source' or 'colors'",
                palette_def.name
            )));
        }
    }

    Ok(())
}

pub fn resolve_input_files(base_dir: &Path, patterns: &[String]) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    for pattern in patterns {
        let full_pattern = base_dir.join(pattern);
        let full_pattern_str = full_pattern.to_string_lossy();

        let matches: Vec<_> = glob::glob(&full_pattern_str)
            .map_err(|e| PipelineError::Config(format!("Invalid glob pattern '{}': {}", pattern, e)))?
            .filter_map(|entry| entry.ok())
            .filter(|path| path.is_file())
            .collect();

        if matches.is_empty() {
            return Err(PipelineError::NoFilesMatched(pattern.clone()));
        }

        files.extend(matches);
    }

    files.sort();
    files.dedup();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let yaml = r#"
project:
  name: "test-project"
"#;
        let config: Config = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.project.name, "test-project");
        assert!(config.sheets.is_empty());
        assert!(config.animations.is_empty());
    }

    #[test]
    fn test_parse_full_config() {
        let yaml = r#"
project:
  name: "dungeon-crawler"
  input_dir: "./raw"
  output_dir: "./dist/assets"

defaults:
  scale_factors: [1, 2, 4]
  padding: 1
  power_of_two: true

sheets:
  - name: "ui-elements"
    inputs: ["ui/icons/*.png"]
    output_formats: ["phaser", "css"]
    padding: 2

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
        let config: Config = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.project.name, "dungeon-crawler");
        assert_eq!(config.sheets.len(), 1);
        assert_eq!(config.animations.len(), 1);
        assert_eq!(config.scaling.factors, vec![1, 2, 4]);
    }

    #[test]
    fn test_validate_empty_project_name() {
        let yaml = r#"
project:
  name: ""
"#;
        let config: Config = serde_yaml_ng::from_str(yaml).unwrap();
        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_sheet_no_inputs() {
        let yaml = r#"
project:
  name: "test"
sheets:
  - name: "empty"
    inputs: []
"#;
        let config: Config = serde_yaml_ng::from_str(yaml).unwrap();
        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_field_rejected() {
        let yaml = r#"
project:
  name: "test"
  unknown_field: true
"#;
        let result: std::result::Result<Config, _> = serde_yaml_ng::from_str(yaml);
        assert!(result.is_err());
    }
}
