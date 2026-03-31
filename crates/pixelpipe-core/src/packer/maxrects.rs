/// MaxRects-BSSF (Best Short Side Fit) bin packing algorithm.
///
/// Places rectangles into a fixed-size bin by maintaining a list of free
/// rectangles. For each sprite, finds the free rect where the shorter
/// leftover side is minimized. Never rotates sprites (pixel art integrity).

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Placement {
    pub id: usize,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct PackResult {
    pub placements: Vec<Placement>,
    pub width: u32,
    pub height: u32,
}

pub struct MaxRectsPacker {
    max_width: u32,
    max_height: u32,
    padding: u32,
    power_of_two: bool,
}

impl MaxRectsPacker {
    pub fn new(max_width: u32, max_height: u32, padding: u32, power_of_two: bool) -> Self {
        Self {
            max_width,
            max_height,
            padding,
            power_of_two,
        }
    }

    /// Pack a list of (id, width, height) rectangles.
    /// Returns placements with coordinates for each sprite, or an error message.
    pub fn pack(&self, mut rects: Vec<(usize, u32, u32)>) -> Result<PackResult, String> {
        if rects.is_empty() {
            return Ok(PackResult {
                placements: vec![],
                width: 0,
                height: 0,
            });
        }

        // Sort by area descending (largest first) for better packing
        rects.sort_by(|a, b| (b.1 * b.2).cmp(&(a.1 * a.2)));

        // Inflate dimensions by padding
        let padded: Vec<(usize, u32, u32)> = rects
            .iter()
            .map(|&(id, w, h)| (id, w + self.padding * 2, h + self.padding * 2))
            .collect();

        // Check that no single sprite exceeds max dimensions
        for &(id, w, h) in &padded {
            if w > self.max_width || h > self.max_height {
                return Err(format!(
                    "Sprite {} ({}x{} with padding) exceeds max sheet size {}x{}",
                    id,
                    w,
                    h,
                    self.max_width,
                    self.max_height
                ));
            }
        }

        let mut free_rects = vec![Rect {
            x: 0,
            y: 0,
            width: self.max_width,
            height: self.max_height,
        }];

        let mut placements = Vec::with_capacity(padded.len());

        for &(id, w, h) in &padded {
            let best = find_best_short_side_fit(&free_rects, w, h);

            let (fx, fy) = match best {
                Some(pos) => pos,
                None => {
                    return Err(format!(
                        "Could not fit sprite {} ({}x{}) into {}x{} atlas",
                        id, w, h, self.max_width, self.max_height
                    ));
                }
            };

            // Record placement with padding offset
            placements.push(Placement {
                id,
                x: fx + self.padding,
                y: fy + self.padding,
                width: w - self.padding * 2,
                height: h - self.padding * 2,
            });

            // Split free rectangles around the placed rect
            let placed = Rect {
                x: fx,
                y: fy,
                width: w,
                height: h,
            };
            split_free_rects(&mut free_rects, &placed);
            prune_contained(&mut free_rects);
        }

        // Compute tight bounding box
        let mut used_w = 0u32;
        let mut used_h = 0u32;
        for p in &placements {
            // Account for the padding around the sprite
            let right = p.x + p.width + self.padding;
            let bottom = p.y + p.height + self.padding;
            used_w = used_w.max(right);
            used_h = used_h.max(bottom);
        }

        if self.power_of_two {
            used_w = next_power_of_two(used_w);
            used_h = next_power_of_two(used_h);
        }

        Ok(PackResult {
            placements,
            width: used_w,
            height: used_h,
        })
    }
}

/// Find the free rectangle where placing (w, h) minimizes the shorter leftover side.
fn find_best_short_side_fit(free_rects: &[Rect], w: u32, h: u32) -> Option<(u32, u32)> {
    let mut best_short_side = u32::MAX;
    let mut best_long_side = u32::MAX;
    let mut best_pos = None;

    for fr in free_rects {
        if w <= fr.width && h <= fr.height {
            let leftover_x = fr.width - w;
            let leftover_y = fr.height - h;
            let short_side = leftover_x.min(leftover_y);
            let long_side = leftover_x.max(leftover_y);

            if short_side < best_short_side
                || (short_side == best_short_side && long_side < best_long_side)
            {
                best_short_side = short_side;
                best_long_side = long_side;
                best_pos = Some((fr.x, fr.y));
            }
        }
    }

    best_pos
}

/// Split all free rectangles that overlap with the placed rect,
/// producing new free rects from the non-overlapping portions.
fn split_free_rects(free_rects: &mut Vec<Rect>, placed: &Rect) {
    let mut new_rects = Vec::new();

    free_rects.retain(|fr| {
        if !overlaps(fr, placed) {
            return true; // keep — no overlap
        }

        // Right portion
        if placed.x + placed.width < fr.x + fr.width {
            new_rects.push(Rect {
                x: placed.x + placed.width,
                y: fr.y,
                width: (fr.x + fr.width) - (placed.x + placed.width),
                height: fr.height,
            });
        }

        // Left portion
        if placed.x > fr.x {
            new_rects.push(Rect {
                x: fr.x,
                y: fr.y,
                width: placed.x - fr.x,
                height: fr.height,
            });
        }

        // Bottom portion
        if placed.y + placed.height < fr.y + fr.height {
            new_rects.push(Rect {
                x: fr.x,
                y: placed.y + placed.height,
                width: fr.width,
                height: (fr.y + fr.height) - (placed.y + placed.height),
            });
        }

        // Top portion
        if placed.y > fr.y {
            new_rects.push(Rect {
                x: fr.x,
                y: fr.y,
                width: fr.width,
                height: placed.y - fr.y,
            });
        }

        false // remove the original overlapping free rect
    });

    free_rects.append(&mut new_rects);
}

/// Remove any free rectangle that is fully contained within another.
fn prune_contained(free_rects: &mut Vec<Rect>) {
    let len = free_rects.len();
    let mut remove = vec![false; len];

    for i in 0..len {
        if remove[i] {
            continue;
        }
        for j in (i + 1)..len {
            if remove[j] {
                continue;
            }
            if contains(&free_rects[i], &free_rects[j]) {
                remove[j] = true;
            } else if contains(&free_rects[j], &free_rects[i]) {
                remove[i] = true;
                break;
            }
        }
    }

    let mut idx = 0;
    free_rects.retain(|_| {
        let keep = !remove[idx];
        idx += 1;
        keep
    });
}

fn overlaps(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.width && a.x + a.width > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

fn contains(outer: &Rect, inner: &Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

fn next_power_of_two(mut n: u32) -> u32 {
    if n == 0 {
        return 1;
    }
    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_single_sprite() {
        let packer = MaxRectsPacker::new(256, 256, 0, false);
        let result = packer.pack(vec![(0, 16, 16)]).unwrap();
        assert_eq!(result.placements.len(), 1);
        assert_eq!(result.placements[0].x, 0);
        assert_eq!(result.placements[0].y, 0);
        assert_eq!(result.width, 16);
        assert_eq!(result.height, 16);
    }

    #[test]
    fn test_pack_multiple_sprites() {
        let packer = MaxRectsPacker::new(256, 256, 0, false);
        let rects = vec![(0, 16, 16), (1, 16, 16), (2, 32, 32), (3, 8, 8)];
        let result = packer.pack(rects).unwrap();
        assert_eq!(result.placements.len(), 4);

        // Verify no overlaps
        for i in 0..result.placements.len() {
            for j in (i + 1)..result.placements.len() {
                let a = &result.placements[i];
                let b = &result.placements[j];
                assert!(
                    !overlaps(
                        &Rect { x: a.x, y: a.y, width: a.width, height: a.height },
                        &Rect { x: b.x, y: b.y, width: b.width, height: b.height },
                    ),
                    "Sprites {} and {} overlap",
                    a.id,
                    b.id
                );
            }
        }
    }

    #[test]
    fn test_pack_with_padding() {
        let packer = MaxRectsPacker::new(256, 256, 1, false);
        let result = packer.pack(vec![(0, 16, 16), (1, 16, 16)]).unwrap();
        assert_eq!(result.placements.len(), 2);

        // With padding=1, first sprite at (1,1), dimensions still 16x16
        assert_eq!(result.placements[0].width, 16);
        assert_eq!(result.placements[0].height, 16);
        assert_eq!(result.placements[0].x, 1);
        assert_eq!(result.placements[0].y, 1);
    }

    #[test]
    fn test_pack_power_of_two() {
        let packer = MaxRectsPacker::new(256, 256, 0, true);
        let result = packer.pack(vec![(0, 10, 10)]).unwrap();
        assert_eq!(result.width, 16); // next PoT from 10
        assert_eq!(result.height, 16);
    }

    #[test]
    fn test_pack_empty() {
        let packer = MaxRectsPacker::new(256, 256, 0, false);
        let result = packer.pack(vec![]).unwrap();
        assert_eq!(result.placements.len(), 0);
        assert_eq!(result.width, 0);
        assert_eq!(result.height, 0);
    }

    #[test]
    fn test_pack_sprite_too_large() {
        let packer = MaxRectsPacker::new(64, 64, 0, false);
        let result = packer.pack(vec![(0, 128, 128)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_placements_outside_bounds() {
        let packer = MaxRectsPacker::new(128, 128, 1, false);
        let rects: Vec<(usize, u32, u32)> = (0..20).map(|i| (i, 16, 16)).collect();
        let result = packer.pack(rects).unwrap();

        for p in &result.placements {
            assert!(
                p.x + p.width <= result.width,
                "Sprite {} right edge {} exceeds atlas width {}",
                p.id,
                p.x + p.width,
                result.width
            );
            assert!(
                p.y + p.height <= result.height,
                "Sprite {} bottom edge {} exceeds atlas height {}",
                p.id,
                p.y + p.height,
                result.height
            );
        }
    }

    #[test]
    fn test_next_power_of_two() {
        assert_eq!(next_power_of_two(0), 1);
        assert_eq!(next_power_of_two(1), 1);
        assert_eq!(next_power_of_two(2), 2);
        assert_eq!(next_power_of_two(3), 4);
        assert_eq!(next_power_of_two(5), 8);
        assert_eq!(next_power_of_two(16), 16);
        assert_eq!(next_power_of_two(17), 32);
        assert_eq!(next_power_of_two(100), 128);
        assert_eq!(next_power_of_two(255), 256);
        assert_eq!(next_power_of_two(256), 256);
    }
}
