use image::{ImageBuffer, Rgb};
use plotters::prelude::*;
use std::f64::consts::PI;
use syl_scr_common::models::VestibuleUserRecord;

pub fn generate_personality_chart(
    record: &VestibuleUserRecord,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let width = 800;
    let height = 800;
    let mut pixel_buffer = vec![0u8; (width * height * 3) as usize];

    {
        let root =
            BitMapBackend::with_buffer(&mut pixel_buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE)?;

        let center = (width as i32 / 2, height as i32 / 2);
        let radius = 300.0;

        let title = format!("{}'s Personality (HEXACO)", record.discord_username);
        root.titled(&title, ("sans-serif", 40))?;

        let traits = [
            "Honesty-Humility",
            "Emotionality",
            "Extraversion",
            "Agreeableness",
            "Conscientiousness",
            "Openness",
        ];

        let values = [
            record.personality.honesty_humility,
            record.personality.emotionality,
            record.personality.extraversion,
            record.personality.agreeableness,
            record.personality.conscientiousness,
            record.personality.openness_to_experience,
        ];

        let n = traits.len();

        // Draw hexagon grid
        for level in 1..=5 {
            let r = radius * (level as f64 / 5.0);

            let mut points = Vec::new();
            for i in 0..n {
                let angle = 2.0 * PI * (i as f64) / (n as f64) - PI / 2.0;
                let x = center.0 as f64 + r * angle.cos();
                let y = center.1 as f64 + r * angle.sin();
                points.push((x as i32, y as i32));
            }
            points.push(points[0]);

            root.draw(&PathElement::new(points, RGBColor(200, 200, 200)))?;
        }

        // Draw axis lines + labels
        for i in 0..n {
            let angle = 2.0 * PI * (i as f64) / (n as f64) - PI / 2.0;

            let x = center.0 as f64 + radius * angle.cos();
            let y = center.1 as f64 + radius * angle.sin();

            root.draw(&PathElement::new(vec![center, (x as i32, y as i32)], BLACK))?;

            let label_x = center.0 as f64 + (radius + 30.0) * angle.cos();
            let label_y = center.1 as f64 + (radius + 30.0) * angle.sin();

            root.draw(&Text::new(
                traits[i],
                (label_x as i32, label_y as i32),
                ("sans-serif", 20).into_font(),
            ))?;
        }

        // Compute radar polygon
        let mut data_points = Vec::new();
        for i in 0..n {
            let angle = 2.0 * PI * (i as f64) / (n as f64) - PI / 2.0;
            let r = radius * values[i];

            let x = center.0 as f64 + r * angle.cos();
            let y = center.1 as f64 + r * angle.sin();

            data_points.push((x as i32, y as i32));
        }
        data_points.push(data_points[0]);

        root.draw(&Polygon::new(data_points.clone(), BLUE.mix(0.4).filled()))?;

        root.draw(&PathElement::new(data_points, &BLUE))?;

        root.present()?;
    }

    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, pixel_buffer)
        .ok_or("Failed to create image buffer")?;

    let mut cursor = std::io::Cursor::new(Vec::new());
    img.write_to(&mut cursor, image::ImageFormat::Png)?;

    Ok(cursor.into_inner())
}
