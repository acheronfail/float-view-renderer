use std::f64::consts::PI;

use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::ttf::Font;

use crate::err::Result;
use crate::{Context, WIDTH};

#[derive(Default)]
pub enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
}

pub struct Text {
    text: String,
    alignment: TextAlignment,
    color: Color,
}

impl Text {
    pub fn new(text: &str) -> Self {
        Text {
            text: String::from(text),
            alignment: TextAlignment::Left,
            color: Color::WHITE,
        }
    }

    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn render(&self, ctx: &mut Context, x: f64, y: f64) -> Result<(u32, u32)> {
        self.render_with_font(ctx, x, y, ctx.font_regular)
    }

    pub fn render_with_font(
        &self,
        ctx: &mut Context,
        x: f64,
        y: f64,
        font: &Font,
    ) -> Result<(u32, u32)> {
        let surface = font.render(&self.text).blended(self.color)?;
        let texture = ctx.tex_creator.create_texture_from_surface(&surface)?;
        let target = Rect::new(
            match self.alignment {
                TextAlignment::Left => x as i32,
                TextAlignment::Center => (x - surface.width() as f64 / 2.0) as i32,
                TextAlignment::Right => (x - surface.width() as f64) as i32,
            },
            (y - surface.height() as f64 / 2.0) as i32,
            surface.width(),
            surface.height(),
        );
        ctx.canvas.copy(&texture, None, Some(target))?;

        Ok((surface.width(), surface.height()))
    }
}

pub struct LabelValue {
    label: String,
    value: String,
    color: Color,
}

impl LabelValue {
    pub fn new(label: &str, value: &str) -> Self {
        LabelValue {
            label: String::from(label),
            value: String::from(value),
            color: Color::WHITE,
        }
    }

    pub fn with_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }

    pub fn render(&self, ctx: &mut Context, y: f64) -> Result<(u32, u32)> {
        let padding = 25.0;

        Text::new(&self.label)
            .with_color(self.color)
            .render(ctx, padding, y)?;
        Text::new(&self.value)
            .with_color(self.color)
            .with_alignment(TextAlignment::Right)
            .render(ctx, WIDTH as f64 - padding, y)?;

        Ok((WIDTH, 40))
    }
}

pub struct Speedo {
    pub title: String,
    pub value: String,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub color: Color,
}

impl Default for Speedo {
    fn default() -> Self {
        Speedo {
            title: String::from("Speedo"),
            value: String::from("xy.z units"),
            min: 0.0,
            max: 100.0,
            step: 10.0,
            color: Color::WHITE,
        }
    }
}

impl Speedo {
    pub fn render(&self, ctx: &mut Context, position: f64, y: f64) -> Result<(u32, u32)> {
        let total = self.max - self.min;
        let arc_color = Color::RGB(255, 255, 255);

        // arc
        let arc_center_x = (WIDTH / 2) as f64;
        let arc_center_y = y + 150.0;
        let arc_radius = 150.0;
        let arc_start_angle = PI;
        let arc_end_angle = 0.0;

        for i in 0..5 as i16 {
            ctx.canvas.arc(
                arc_center_x as i16,
                arc_center_y as i16,
                (arc_radius + i as f64) as i16,
                (arc_start_angle * 180.0 / PI) as i16,
                (arc_end_angle * 180.0 / PI) as i16,
                arc_color,
            )?;
        }

        // draw ticks
        let tick_length = 20.0;
        let num_ticks = (total / self.step).floor() as i32;

        for i in 0..=num_ticks {
            let angle =
                arc_start_angle + (arc_end_angle - arc_start_angle) * (i as f64 / num_ticks as f64);
            let inner_x = arc_center_x + (arc_radius - tick_length) * angle.cos();
            let inner_y = arc_center_y - (arc_radius - tick_length) * angle.sin();
            let outer_x = arc_center_x + arc_radius * angle.cos();
            let outer_y = arc_center_y - arc_radius * angle.sin();

            ctx.canvas.thick_line(
                inner_x as i16,
                inner_y as i16,
                outer_x as i16,
                outer_y as i16,
                2,
                arc_color,
            )?;
        }

        // draw tick labels
        let label_radius = arc_radius - tick_length - 20.0;
        for i in 0..=num_ticks {
            let angle =
                arc_start_angle + (arc_end_angle - arc_start_angle) * (i as f64 / num_ticks as f64);
            let label_x = arc_center_x + label_radius * angle.cos();
            let label_y = arc_center_y - label_radius * angle.sin();

            let label = format!("{}", i * 10);
            let surface = ctx
                .font_small
                .render(&label)
                .blended(Color::RGB(255, 255, 255))?;
            let texture = ctx.tex_creator.create_texture_from_surface(&surface)?;
            let target = Rect::new(
                (label_x - surface.width() as f64 / 2.0) as i32,
                (label_y - surface.height() as f64 / 2.0) as i32,
                surface.width(),
                surface.height(),
            );
            ctx.canvas.copy(&texture, None, Some(target))?;
        }

        // needle
        {
            let needle_length = 140.0;
            let needle_width = 5.0;
            let needle_angle =
                arc_start_angle + (arc_end_angle - arc_start_angle) * (position as f64 / total);

            let needle_tip_x = arc_center_x + needle_length * needle_angle.cos();
            let needle_tip_y = arc_center_y - needle_length * needle_angle.sin();

            let base_angle1 = needle_angle + PI / 2.0;
            let base_angle2 = needle_angle - PI / 2.0;

            let base_x1 = arc_center_x + needle_width * base_angle1.cos();
            let base_y1 = arc_center_y - needle_width * base_angle1.sin();
            let base_x2 = arc_center_x + needle_width * base_angle2.cos();
            let base_y2 = arc_center_y - needle_width * base_angle2.sin();

            ctx.canvas.filled_trigon(
                needle_tip_x as i16,
                needle_tip_y as i16,
                base_x1 as i16,
                base_y1 as i16,
                base_x2 as i16,
                base_y2 as i16,
                self.color,
            )?;
        }

        Text::new(&self.title)
            .with_alignment(TextAlignment::Center)
            .render(ctx, arc_center_x, arc_center_y - 50.0)?;
        Text::new(&self.value)
            .with_alignment(TextAlignment::Center)
            .render(ctx, arc_center_x, arc_center_y + 50.0)?;

        Ok((WIDTH, 250))
    }
}

pub struct TextTitle {
    title: String,
    color: Color,
}

impl TextTitle {
    pub fn new(title: &str) -> Self {
        TextTitle {
            title: String::from(title),
            color: Color::WHITE,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn render(&self, ctx: &mut Context, y: f64) -> Result<(u32, u32)> {
        let (_, h) = Text::new(&self.title)
            .with_color(self.color)
            .with_alignment(TextAlignment::Left)
            .render_with_font(ctx, 20.0, y, ctx.font_title)?;

        ctx.canvas.thick_line(
            10,
            (y + h as f64 / 2.0 + 5.0) as i16,
            (WIDTH - 10) as i16,
            (y + h as f64 / 2.0 + 5.0) as i16,
            2,
            self.color,
        )?;

        Ok((WIDTH, 40))
    }
}

pub struct List {
    title: String,
    items: Vec<LabelValue>,
    color: Option<Color>,
}

impl List {
    pub fn new(title: &str, items: Vec<LabelValue>) -> Self {
        List {
            title: String::from(title),
            items,
            color: None,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn render(&mut self, ctx: &mut Context, y: f64) -> Result<(u32, u32)> {
        let top = y as u32;
        let mut offset = y as u32;

        offset += TextTitle::new(&self.title)
            .with_color(Color::GREY)
            .render(ctx, offset as f64)?
            .1;

        for item in &mut self.items {
            offset += match self.color {
                Some(color) => item.with_color(color).render(ctx, offset as f64)?.1,
                None => item.render(ctx, offset as f64)?.1,
            };
        }

        Ok((WIDTH, offset - top))
    }
}
