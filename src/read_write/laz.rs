use crate::{AttributeData, NumberOfPoints, PointsBatch};
use las::Read as LasRead;
use nalgebra::{Point3, Vector3};
use std::collections::BTreeMap;
use std::path::Path;

pub struct LazIterator {
    // SAFETY: Reader::from_path opens a BufReader<File> owned by the Reader;
    // no external borrow is held, so 'static is sound for this construction path.
    reader: las::Reader<'static>,
    num_total_points: usize,
    batch_size: usize,
    point_count: usize,
    // Locked in after the first batch with non-zero color to keep scale consistent
    // across all batches (avoids color discontinuities at octree node boundaries).
    is_16bit: Option<bool>,
}

impl LazIterator {
    pub fn from_file(path: impl AsRef<Path>, batch_size: usize) -> Result<Self, las::Error> {
        let reader = las::Reader::from_path(path.as_ref())?;
        // u64 → usize: on 64-bit Linux usize is 8 bytes, lossless for any realistic file.
        let num_total_points = reader.header().number_of_points() as usize;
        Ok(Self {
            reader,
            num_total_points,
            batch_size,
            point_count: 0,
            is_16bit: None,
        })
    }
}

impl NumberOfPoints for LazIterator {
    fn num_points(&self) -> usize {
        self.num_total_points
    }
}

impl Iterator for LazIterator {
    type Item = PointsBatch;

    fn next(&mut self) -> Option<PointsBatch> {
        if self.point_count >= self.num_total_points {
            return None;
        }
        let cur_batch_size = self.batch_size.min(self.num_total_points - self.point_count);

        let mut positions = Vec::with_capacity(cur_batch_size);
        let mut intensities: Vec<f32> = Vec::with_capacity(cur_batch_size);
        let mut raw_r: Vec<u16> = Vec::new();
        let mut raw_g: Vec<u16> = Vec::new();
        let mut raw_b: Vec<u16> = Vec::new();
        let mut has_color_field = false;
        let mut exhausted = false;

        for _ in 0..cur_batch_size {
            match self.reader.read() {
                Some(Ok(point)) => {
                    positions.push(Point3::new(point.x, point.y, point.z));
                    intensities.push(point.intensity as f32 / 65535.0);
                    if let Some(color) = point.color {
                        has_color_field = true;
                        raw_r.push(color.red);
                        raw_g.push(color.green);
                        raw_b.push(color.blue);
                    }
                }
                Some(Err(e)) => {
                    eprintln!("laz: read error at point {}: {}", self.point_count, e);
                    exhausted = true;
                    break;
                }
                None => {
                    exhausted = true;
                    break;
                }
            }
        }

        if positions.is_empty() {
            self.point_count = self.num_total_points;
            return None;
        }
        self.point_count += positions.len();
        if exhausted {
            self.point_count = self.num_total_points;
        }

        let max_raw = raw_r.iter().chain(&raw_g).chain(&raw_b).copied().max().unwrap_or(0);

        let color_data: Vec<Vector3<u8>> = if has_color_field && max_raw > 0 {
            // Some scanners store 8-bit values in the u16 field; others use the full 16-bit range.
            // Detect on the first non-zero color batch and reuse for all subsequent batches.
            let is_16bit = *self.is_16bit.get_or_insert(max_raw > 255);
            raw_r
                .iter()
                .zip(raw_g.iter())
                .zip(raw_b.iter())
                .map(|((&r, &g), &b)| {
                    if is_16bit {
                        Vector3::new((r >> 8) as u8, (g >> 8) as u8, (b >> 8) as u8)
                    } else {
                        Vector3::new(r as u8, g as u8, b as u8)
                    }
                })
                .collect()
        } else {
            // No RGB in file — stretch intensity to full 0-255 range for grayscale display.
            let (min_i, max_i) = intensities
                .iter()
                .fold((f32::MAX, f32::MIN), |(mn, mx), &v| (mn.min(v), mx.max(v)));
            let range = max_i - min_i;
            intensities
                .iter()
                .map(|&v| {
                    let normalized = if range > 1e-6 {
                        (v - min_i) / range
                    } else {
                        0.5
                    };
                    let gray = (normalized * 255.0) as u8;
                    Vector3::new(gray, gray, gray)
                })
                .collect()
        };

        let mut attributes = BTreeMap::new();
        attributes.insert("intensity".to_string(), AttributeData::F32(intensities));
        attributes.insert("color".to_string(), AttributeData::U8Vec3(color_data));

        Some(PointsBatch {
            position: positions,
            attributes,
        })
    }
}
