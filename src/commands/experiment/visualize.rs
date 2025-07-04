use std::{collections::HashMap, time::Duration};

use ab_glyph::{Font, FontArc, FontRef, ScaleFont as _};
use anyhow::Context as _;
use image::{ImageBuffer, Rgb, RgbImage, Rgba};
use imageproc::{
    drawing::{draw_filled_rect, draw_filled_rect_mut},
    rect::Rect,
};
use itertools::Itertools;
use serde_json::{Map, Value};

use crate::{
    config::{EtnaConfig, ExperimentConfig},
    experiment::Test,
    property,
    store::{Metric, Store},
    strategy,
    workload::Language,
};

/// Visualize the results of an experiment for a given set of tests.
pub fn invoke(
    experiment_name: Option<String>,
    figure_name: String,
    tests: Vec<String>,
    groupby: Vec<String>,
    aggby: Vec<String>,
    mut buckets: Vec<f64>,
) -> anyhow::Result<()> {
    log::trace!("visualizing experiment with name '{:?}'", experiment_name);
    let etna_config = EtnaConfig::get_etna_config()?;

    if tests.is_empty() {
        anyhow::bail!("No tests provided. Please specify at least one test to run.");
    }

    let experiment_config = match experiment_name {
        Some(name) => ExperimentConfig::from_etna_config(&name, &etna_config).context(format!(
            "Failed to get experiment config for '{}'",
            name
        )),
        None => ExperimentConfig::from_current_dir().context("No experiment name is provided, and the current directory is not an experiment directory"),
    }?;

    let store = Store::load(&experiment_config.store)?;

    let experiment = store.get_experiment_by_name(&experiment_config.name)?;

    let tests = tests
        .iter()
        .flat_map(|test| {
            let test_path = experiment
                .path
                .join("tests")
                .join(test)
                .with_extension("json");
            let test: Vec<Test> =
                serde_json::from_str(&std::fs::read_to_string(test_path).unwrap()).unwrap();
            test
        })
        .collect::<Vec<Test>>();

    log::trace!("Loaded {} tests for visualization", tests.len());

    let metrics = tests
        .iter()
        .flat_map(|test| {
            store.metrics.iter().filter_map(|m| {
                let data = m.data.as_object().unwrap();

                let language = data.get("language").and_then(serde_json::Value::as_str)?;
                let workload = data.get("workload").and_then(serde_json::Value::as_str)?;
                let mutations = data
                    .get("mutations")
                    .and_then(serde_json::Value::as_array)?;
                let strategy = data.get("strategy").and_then(serde_json::Value::as_str)?;
                let property = data.get("property").and_then(serde_json::Value::as_str)?;

                let result = test.language == language
                    && test.workload == workload
                    && &test.mutations == mutations
                    && test
                        .tasks
                        .contains(&(strategy.to_string(), property.to_string()));

                if result {
                    Some(m)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();

    log::trace!("Aggregated metrics by: {:#?}", aggby);

    log::trace!("metrics: {:#?}", &metrics[..5.min(metrics.len())]);

    let aggs = aggby
        .iter()
        .map(|g| {
            metrics
                .iter()
                .map(|m| {
                    m.data
                        .get(g)
                        .expect(format!("Aggby field '{g}' not found").as_str())
                })
                .unique()
                .collect::<Vec<_>>()
        })
        .multi_cartesian_product()
        .collect::<Vec<Vec<_>>>();

    log::trace!("Aggregations: {:#?}", aggs);

    let agg_metrics = aggs
        .iter()
        .map(|agg| {
            let agg_metrics = metrics
                .iter()
                .filter(|m| {
                    aggby
                        .iter()
                        .enumerate()
                        .all(|(i, g)| m.data.get(g).map_or(false, |v| agg[i] == v))
                })
                .collect::<Vec<_>>();
            log::trace!("Group: {:#?}", agg);
            log::trace!("Number of metrics in agg: {}", agg_metrics.len());
            let sums: (f64, f64, f64, f64) =
                agg_metrics.iter().fold((0.0, 0.0, 0.0, 0.0), |mut acc, m| {
                    acc.0 = m
                        .data
                        .get("discards")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(0.0);
                    acc.1 += m
                        .data
                        .get("tests")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(0.0);
                    acc.2 += m
                        .data
                        .get("shrinks")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(0.0);
                    acc.3 += m
                        .data
                        .get("time")
                        .and_then(serde_json::Value::as_str)
                        .into_iter()
                        .flat_map(parse_duration::parse)
                        .next()
                        .and_then(|d| Some(d.as_secs_f64()))
                        .expect("Failed to parse time");
                    acc
                });
            let avgs = (
                sums.0 / agg_metrics.len() as f64,
                sums.1 / agg_metrics.len() as f64,
                sums.2 / agg_metrics.len() as f64,
                sums.3 / agg_metrics.len() as f64,
            );

            log::debug!(
                "Aggregated metrics for group {:?}: \n\
            Discards: {:.2}, Tests: {:.2}, Shrinks: {:.2}, Time: {:.4} seconds",
                agg,
                avgs.0,
                avgs.1,
                avgs.2,
                avgs.3
            );

            let data = serde_json::json!({
                "language": agg[0],
                "workload": agg[1],
                "strategy": agg[2],
                "property": agg[3],
                "mutations": agg[4],
                "discards": avgs.0,
                "tests": avgs.1,
                "shrinks": avgs.2,
                "time": avgs.3,
            });

            data.as_object().unwrap().to_owned()
        })
        .collect::<Vec<_>>();

    log::trace!("Aggregated metrics: {:#?}", agg_metrics);

    log::trace!("Number of aggregated metrics: {}", agg_metrics.len());

    log::trace!("Groupby fields: {:#?}", groupby);

    let groups = groupby
        .iter()
        .map(|g| {
            agg_metrics
                .iter()
                .map(|m| {
                    m.get(g)
                        .expect(format!("Groupby field '{g}' not found").as_str())
                })
                .unique()
                .collect::<Vec<_>>()
        })
        .multi_cartesian_product()
        .collect::<Vec<Vec<_>>>();

    log::trace!("groups: {:?}", groups);

    let width = 500.0;
    let height = 200.0;
    let margin = 20.0;
    let bucket_height = ((height - margin) / (groups.len() as f64) - margin) as f64;

    let mut image = RgbImage::new(width as u32, height as u32);

    draw_filled_rect_mut(
        &mut image,
        Rect::at(0, 0).of_size(width as u32, height as u32),
        Rgb([255, 255, 255]),
    );

    let colors = [
        Rgb([0x00, 0x00, 0x00]), // black
        Rgb([0x90, 0x0D, 0x0D]), // red
        Rgb([0xDC, 0x5F, 0x00]), // orange
        Rgb([0x24, 0x37, 0x63]), // blue
        Rgb([0x43, 0x6E, 0x4F]), // green
        Rgb([0x47, 0x09, 0x38]), // purple
        Rgb([0xD6, 0x1C, 0x4E]), // pink
        Rgb([0x33, 0x47, 0x56]), // dark blue
        Rgb([0x29, 0x00, 0x01]), // dark brown
        Rgb([0x00, 0x00, 0x00]), // black
    ];

    for (i, group) in groups.iter().enumerate() {
        log::trace!("Processing group {i}: {:?}", group);
        let group_metrics = agg_metrics
            .iter()
            .filter(|m| {
                groupby
                    .iter()
                    .enumerate()
                    .all(|(i, g)| m.get(g).map_or(false, |v| group[i] == v))
            })
            .collect::<Vec<_>>();

        if buckets[0] != 0.0 {
            buckets.insert(0, 0.0);
        }
        buckets.push(f64::INFINITY);

        // create buckets between b[0-1], b[1-2], ..., b[n-1-n]
        let mut buckets: Vec<((f64, f64), Vec<Map<String, Value>>)> = buckets
            .windows(2)
            .map(|w| ((w[0], w[1]), vec![]))
            .collect::<Vec<_>>();

        for metric in &group_metrics {
            let time = metric
                .get("time")
                .and_then(serde_json::Value::as_f64)
                .expect("Time not found in metric data");

            for ((start, end), metrics) in &mut buckets {
                if time >= *start && time < *end {
                    metrics.push((*metric).clone());
                }
            }
        }

        println!(
            "Group: {:?}\nBuckets: {:?}",
            group,
            buckets
                .iter()
                .map(|((start, end), values)| format!(
                    "{:.2} - {:.2}: {:?}",
                    start,
                    end,
                    values
                        .iter()
                        .map(|m| m
                            .get("time")
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(0.0))
                        .collect::<Vec<_>>()
                ))
                .collect::<Vec<_>>()
        );

        let buckets = buckets
            .into_iter()
            .map(|((start, end), values)| {
                (
                    (start, end),
                    values
                        .iter()
                        .map(|m| {
                            m.get("time")
                                .and_then(serde_json::Value::as_f64)
                                .unwrap_or(0.0)
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();

        let cfg = BucketDrawConfig {
            width,
            height,
            margin,
            bucket_height,
            bucket_index: i,
            fill_color: colors[i].clone(),
        };
        draw_buckets_line(&mut image, buckets, cfg);
    }

    let path = experiment.path.join("figures").join("buckets.png");

    image.save(path).expect("Failed to save image");

    Ok(())
}

pub struct BucketDrawConfig {
    pub width: f64,
    pub height: f64,
    pub margin: f64,
    pub bucket_height: f64,
    pub bucket_index: usize,
    pub fill_color: Rgb<u8>,
}

fn luma(r: u8, g: u8, b: u8) -> f64 {
    0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64
}

fn text_color(fill_color: Rgb<u8>) -> Rgb<u8> {
    // Use a simple heuristic to determine text color based on the fill color
    // If the fill color is light, use dark text; if dark, use light text
    let luma_value = luma(fill_color[0], fill_color[1], fill_color[2]);

    if luma_value > 100.0 {
        Rgb([0u8, 0u8, 0u8]) // Dark text for light background
    } else {
        Rgb([255u8, 255u8, 255u8]) // Light text for dark background
    }
}

fn rendered_text_width_and_height(text: &str, font: &FontRef, font_size: f64) -> (f64, f64) {
    let scaled_font = font.as_scaled(font_size as f32);
    let mut width = 0.0;

    for ch in text.chars() {
        let glyph = scaled_font.scaled_glyph(ch);
        width += scaled_font.h_advance(glyph.id);
    }

    let height = scaled_font.height() as f64;

    (width as f64, height)
}

fn draw_buckets_line(
    image: &mut RgbImage,
    buckets: Vec<((f64, f64), Vec<f64>)>,
    mut cfg: BucketDrawConfig,
) {
    let total_num_bugs = buckets
        .iter()
        .map(|(_, metrics)| metrics.len())
        .sum::<usize>();

    let single_bug_width = (cfg.width - 2.0 * cfg.margin) / total_num_bugs as f64;

    let mut x = cfg.margin;
    let y = cfg.margin + cfg.bucket_index as f64 * (cfg.bucket_height + cfg.margin);

    let scale = cfg.bucket_height * 0.4;

    let color_moves = [
        (240u8 - cfg.fill_color[0]) / (buckets.len() - 1) as u8,
        (240u8 - cfg.fill_color[1]) / (buckets.len() - 1) as u8,
        (240u8 - cfg.fill_color[2]) / (buckets.len() - 1) as u8,
    ];

    let font = ab_glyph::FontRef::try_from_slice(include_bytes!(
        "../../../assets/SourceCodePro-Medium.ttf"
    ))
    .expect("Failed to load font");

    for ((_, end), metrics) in buckets {
        if metrics.is_empty() {
            cfg.fill_color[0] = cfg.fill_color[0].saturating_add(color_moves[0]);
            cfg.fill_color[1] = cfg.fill_color[1].saturating_add(color_moves[1]);
            cfg.fill_color[2] = cfg.fill_color[2].saturating_add(color_moves[2]);

            continue; // Skip empty buckets
        }

        // Calculate the width of the bucket based on the number of metrics
        let bucket_width = metrics.len() as f64 * single_bug_width;

        // Calculate the text color based on the fill color
        let text_color = text_color(cfg.fill_color);

        log::trace!(
            "Filling rectangle from ({}, {}) to ({}, {}) with color {:?}",
            x,
            y,
            x + bucket_width,
            y + cfg.bucket_height,
            cfg.fill_color
        );

        let rect =
            Rect::at(x as i32, y as i32).of_size(bucket_width as u32, cfg.bucket_height as u32);

        draw_filled_rect_mut(image, rect, cfg.fill_color);

        // Write the bucket label
        let label = format!("{}", metrics.len());
        let (text_width, text_height) = rendered_text_width_and_height(&label, &font, scale);

        if text_width > bucket_width {
            log::trace!(
                "Text width ({}) exceeds bucket width ({}), not drawing text",
                text_width,
                bucket_width
            );
        } else {
            let text_x = x + (bucket_width / 2.0) - (text_width / 2.0);
            let text_y = y + (cfg.bucket_height / 2.0) - (text_height / 2.0);
            log::trace!(
                "Drawing text '{}' at ({}, {}) with color {:?}",
                label,
                text_x,
                text_y,
                text_color
            );
            imageproc::drawing::draw_text_mut(
                image,
                text_color,
                text_x as i32,
                text_y as i32,
                scale as f32,
                &font,
                &label,
            );
        }
        cfg.fill_color[0] = cfg.fill_color[0].saturating_add(color_moves[0]);
        cfg.fill_color[1] = cfg.fill_color[1].saturating_add(color_moves[1]);
        cfg.fill_color[2] = cfg.fill_color[2].saturating_add(color_moves[2]);

        x += bucket_width;
    }
}
