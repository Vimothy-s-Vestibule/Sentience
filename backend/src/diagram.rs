use image::{ImageBuffer, Rgb};
use plotters::prelude::*;
use std::io::Cursor;
use syl_scr_common::models::VestibuleUserRecord;

pub fn generate_personality_chart(
    record: &VestibuleUserRecord,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let width = 800;
    let height = 600;
    let mut pixel_buffer = vec![0u8; (width * height * 3) as usize];

    {
        let root =
            BitMapBackend::with_buffer(&mut pixel_buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE)?;

        let title = format!("{}'s Personality (HEXACO)", record.discord_username);
        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 40).into_font())
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(200)
            .build_cartesian_2d(0.0..1.0f64, 0usize..6usize)?;

        let trait_names = [
            "Honesty-Humility",
            "Emotionality",
            "Extraversion",
            "Agreeableness",
            "Conscientiousness",
            "Openness",
        ];

        chart
            .configure_mesh()
            .x_desc("Score (0.0 - 1.0)")
            .x_labels(10)
            .y_labels(6)
            .y_label_formatter(&|y| {
                if *y < trait_names.len() {
                    trait_names[(*y)].to_string()
                } else {
                    "".to_string()
                }
            })
            .draw()?;

        let values = vec![
            record.personality.honesty_humility.unwrap_or(0.0),
            record.personality.emotionality.unwrap_or(0.0),
            record.personality.extraversion.unwrap_or(0.0),
            record.personality.agreeableness.unwrap_or(0.0),
            record.personality.conscientiousness.unwrap_or(0.0),
            record.personality.openness_to_experience.unwrap_or(0.0),
        ];

        chart.draw_series(
            Histogram::horizontal(&chart)
                .style(BLUE.filled())
                .data(values.into_iter().enumerate()),
        )?;

        root.present()?;
    }

    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, pixel_buffer)
        .ok_or("Failed to create image buffer")?;

    let mut out_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut out_bytes);
    img.write_to(&mut cursor, image::ImageFormat::Png)?;

    Ok(out_bytes)
}
