pub mod gif;
pub mod strip;

use crate::config::resolve_input_files;
use crate::config::schema::{AnimationConfig, AnimationOutputType, SortOrder, StripDirection};
use crate::error::{PipelineError, Result};
use crate::pipeline::{AnimationResult, PipelineContext, PipelinePhase};
use std::path::Path;

pub struct AnimationPhase;

impl PipelinePhase for AnimationPhase {
    fn name(&self) -> &str {
        "animation-assembly"
    }

    fn execute(&self, ctx: &mut PipelineContext) -> Result<()> {
        if ctx.config.animations.is_empty() {
            log::info!("No animations configured, skipping animation phase");
            return Ok(());
        }

        let resolve_dir = ctx.base_dir.join(&ctx.config.project.input_dir);
        let output_dir = ctx.base_dir.join(&ctx.config.project.output_dir);
        let dry_run = ctx.options.dry_run;
        if !dry_run {
            std::fs::create_dir_all(&output_dir).map_err(|e| PipelineError::Io {
                path: output_dir.clone(),
                source: e,
            })?;
        }

        for anim_config in &ctx.config.animations {
            log::info!("Assembling animation '{}'", anim_config.name);
            let result = assemble_animation(anim_config, &resolve_dir, &output_dir, dry_run)?;
            ctx.animations
                .insert(anim_config.name.clone(), result);
        }

        Ok(())
    }
}

fn assemble_animation(
    config: &AnimationConfig,
    resolve_dir: &Path,
    output_dir: &Path,
    dry_run: bool,
) -> Result<AnimationResult> {
    // Load frames
    let mut frame_images = Vec::new();
    for frame_source in &config.frames {
        let mut files = resolve_input_files(resolve_dir, &[frame_source.pattern.clone()])?;

        // Sort frames
        match frame_source.sort {
            SortOrder::Natural => natural_sort(&mut files),
            SortOrder::Alphabetical => files.sort(),
        }

        for file_path in &files {
            let img = image::open(file_path)
                .map_err(|e| PipelineError::Image {
                    path: file_path.clone(),
                    source: e,
                })?
                .to_rgba8();
            frame_images.push(img);
        }
    }

    if frame_images.is_empty() {
        return Err(PipelineError::Animation(format!(
            "No frames found for animation '{}'",
            config.name
        )));
    }

    let frame_w = frame_images[0].width();
    let frame_h = frame_images[0].height();
    let frame_count = frame_images.len() as u32;

    // Build timing array
    let delays = build_delays(&config.timing, frame_count)?;

    let mut result = AnimationResult {
        strip_image: None,
        gif_data: None,
        frame_count,
        frame_width: frame_w,
        frame_height: frame_h,
        timing: delays.clone(),
    };

    // Process each output type
    for output in &config.outputs {
        match output.output_type {
            AnimationOutputType::Gif => {
                let loop_anim = output.loop_animation.unwrap_or(true);
                let gif_data = gif::encode_gif(&frame_images, &delays, loop_anim)?;

                if !dry_run {
                    let gif_filename = output
                        .output
                        .clone()
                        .unwrap_or_else(|| format!("{}.gif", config.name));
                    let gif_path = output_dir.join(&gif_filename);
                    if let Some(parent) = gif_path.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| PipelineError::Io {
                            path: parent.to_path_buf(),
                            source: e,
                        })?;
                    }
                    std::fs::write(&gif_path, &gif_data).map_err(|e| PipelineError::Io {
                        path: gif_path.clone(),
                        source: e,
                    })?;
                    log::info!("Wrote {}", gif_path.display());
                }

                result.gif_data = Some(gif_data);
            }
            AnimationOutputType::Strip => {
                let direction = output
                    .direction
                    .clone()
                    .unwrap_or(StripDirection::Horizontal);

                let strip_image = strip::build_strip(&frame_images, &direction)?;

                if !dry_run {
                    let strip_filename = output
                        .output
                        .clone()
                        .unwrap_or_else(|| format!("{}-strip.png", config.name));
                    let strip_path = output_dir.join(&strip_filename);
                    if let Some(parent) = strip_path.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| PipelineError::Io {
                            path: parent.to_path_buf(),
                            source: e,
                        })?;
                    }
                    strip_image
                        .save(&strip_path)
                        .map_err(|e| PipelineError::Image {
                            path: strip_path.clone(),
                            source: e,
                        })?;
                    log::info!("Wrote {}", strip_path.display());

                    if output.metadata.unwrap_or(false) {
                        let meta = strip::build_metadata(
                            &strip_filename,
                            frame_w,
                            frame_h,
                            frame_count,
                            &direction,
                            &delays,
                        );
                        let json = serde_json::to_string_pretty(&meta).map_err(|e| {
                            PipelineError::Output(format!("JSON serialization failed: {}", e))
                        })?;
                        let json_path = strip_path.with_extension("json");
                        std::fs::write(&json_path, &json).map_err(|e| PipelineError::Io {
                            path: json_path.clone(),
                            source: e,
                        })?;
                        log::info!("Wrote {}", json_path.display());
                    }
                }

                result.strip_image = Some(strip_image);
            }
        }
    }

    Ok(result)
}

fn build_delays(
    timing: &crate::config::schema::TimingConfig,
    frame_count: u32,
) -> Result<Vec<u32>> {
    if let Some(ref durations) = timing.durations_ms {
        if durations.len() != frame_count as usize {
            return Err(PipelineError::Animation(format!(
                "durations_ms has {} entries but animation has {} frames",
                durations.len(),
                frame_count
            )));
        }
        Ok(durations.clone())
    } else if let Some(uniform) = timing.frame_duration_ms {
        Ok(vec![uniform; frame_count as usize])
    } else {
        Err(PipelineError::Animation(
            "Animation timing must specify frame_duration_ms or durations_ms".to_string(),
        ))
    }
}

/// Sort file paths using natural ordering (frame_1, frame_2, ..., frame_10).
fn natural_sort(paths: &mut [std::path::PathBuf]) {
    paths.sort_by(|a, b| {
        let a_name = a.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let b_name = b.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        natural_cmp(a_name, b_name)
    });
}

fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(&ac), Some(&bc)) => {
                if ac.is_ascii_digit() && bc.is_ascii_digit() {
                    let a_num = consume_number(&mut a_chars);
                    let b_num = consume_number(&mut b_chars);
                    match a_num.cmp(&b_num) {
                        std::cmp::Ordering::Equal => continue,
                        other => return other,
                    }
                } else {
                    match ac.cmp(&bc) {
                        std::cmp::Ordering::Equal => {
                            a_chars.next();
                            b_chars.next();
                        }
                        other => return other,
                    }
                }
            }
        }
    }
}

fn consume_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> u64 {
    let mut num = 0u64;
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            num = num * 10 + c.to_digit(10).unwrap() as u64;
            chars.next();
        } else {
            break;
        }
    }
    num
}
