use std::cmp::Ordering;

use anyhow::Result;
use image::{GenericImage, GenericImageView, ImageBuffer, Pixel, Primitive, Rgba, RgbaImage};
use imageproc::drawing::{draw_text_mut, text_size};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rusttype::{Font, Scale};

pub struct SetUp {
    font: Font<'static>,
    scale: Scale,
    gif_w: u32,
}

impl SetUp {
    pub fn init(font: Font<'static>) -> Self {
        Self {
            font,
            scale: Scale { x: 0.0, y: 0.0 },
            gif_w: 0,
        }
    }

    pub fn with_dimensions(self, width: u32, height: u32) -> Self {
        Self {
            gif_w: width,
            scale: Scale::uniform(height as f32 / 8.0),
            ..self
        }
    }

    pub fn font(&self) -> &Font<'_> {
        &self.font
    }

    pub fn scale(&self) -> Scale {
        self.scale
    }
}

pub struct TextImage {
    init: SetUp,
    strings: Vec<String>,
    // buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl TextImage {
    pub fn new(init: SetUp, text: &str) -> Result<Self> {
        let text_nlsplit: Vec<_> = text.split('\n').collect();
        let mut split_texts: Vec<Vec<_>> = vec![];
        for str in text_nlsplit {
            split_texts.push(str.split_whitespace().collect());
        }
        let scale = init.scale();

        // let (height, width) = text_size(scale, init.font(), text);
        let strings = Self::wrap_text(
            &Self::sum_until_fit(scale, init.font(), init.gif_w as i32, &split_texts),
            &split_texts,
        );

        Ok(Self { init, strings })
    }

    pub fn render(self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let images: Vec<_> = self
            .strings
            .par_iter()
            .map(|text| self.render_text(text))
            .collect();
        let image = Self::v_concat(&images);
        let image_h = image.height();

        let image = Self::set_bg(image, self.init.gif_w);
        Self::resize(image, self.init.gif_w, image_h as _)
    }

    fn render_text(&self, text: &str) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (text_width, text_height) = text_size(self.init.scale(), self.init.font(), text);
        let y_extension = (text_height as f32 * 1.2) as u32;
        let mut image = RgbaImage::new(text_width as u32, y_extension);
        let y_offset = (image.height() as i32 - text_height) / 2;
        draw_text_mut(
            &mut image,
            Rgba([0u8, 0u8, 0u8, 255u8]),
            0,
            y_offset,
            self.init.scale(),
            self.init.font(),
            text,
        );
        // image.save()
        image
    }

    fn resize(
        image: ImageBuffer<Rgba<u8>, Vec<u8>>,
        t_width: u32,
        image_h: u32,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        image::imageops::resize(
            &image,
            Self::npercent(image.width(), t_width),
            image_h as _,
            image::imageops::FilterType::Gaussian,
        )
    }
    fn set_bg(
        buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
        gif_w: u32,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut bg = blank_buffer_new(gif_w, buffer.height() as _);

        let (x, y) = {
            let (bg_h, bg_w) = (bg.height() as i32, bg.width() as i32);
            let (img_h, img_w) = (buffer.height() as i32, buffer.width() as i32);
            ((bg_w - img_w) / 2, (bg_h - img_h) / 2)
        };
        // dbg!(x, y);

        image::imageops::overlay(&mut bg, &buffer, x as _, y as _);
        bg
    }

    fn v_concat<I, P, S>(images: &[I]) -> ImageBuffer<P, Vec<S>>
    where
        I: GenericImageView<Pixel = P>,
        P: Pixel<Subpixel = S> + 'static,
        S: Primitive + 'static,
    {
        // The height is the sum of all images height.
        let img_height_out: u32 = images.iter().map(image::GenericImageView::height).sum();

        // The final width is the maximum width from the input images.
        let img_width_out: u32 = images
            .iter()
            .map(image::GenericImageView::width)
            .max()
            .unwrap_or(0);

        // Initialize an image buffer with the appropriate size.
        let mut imgbuf = image::ImageBuffer::new(img_width_out, img_height_out);
        let mut accumulated_height = 0;

        // Copy each input image at the correct location in the output image.
        for img in images {
            imgbuf
                .copy_from(img, (img_width_out - img.width()) / 2, accumulated_height)
                .map_err(|e| println!("{}", e))
                .unwrap();
            accumulated_height += img.height();
        }

        imgbuf
    }

    fn sum_until_fit(
        scale: Scale,
        font: &Font,
        image_width: i32,
        split_texts: &[Vec<&str>],
    ) -> Vec<Vec<usize>> {
        let mut split_at = vec![];
        for elem in split_texts {
            let mut accumulator = 0;
            let mut str_widths = vec![];
            for text in elem {
                str_widths.push(text_size(scale, font, text).0);
            }

            let mut tempvec = Vec::with_capacity(str_widths.len() + 1);
            for x in 0..str_widths.len() {
                if str_widths[accumulator..=x]
                    .iter()
                    .sum::<i32>()
                    .cmp(&image_width)
                    == Ordering::Greater
                {
                    tempvec.push(x);
                    accumulator = x;
                }
            }

            tempvec.push(str_widths.len());
            split_at.push(tempvec);
        }
        // split_at.sort_unstable();
        split_at
    }

    fn npercent(current_width: u32, target_width: u32) -> u32 {
        // dbg!(current_width, target_width);
        // dbg!((current_width as f32 * (target_width as f32 / current_width as f32)) as u32);
        (current_width as f32 * (target_width as f32 / current_width as f32)) as u32
    }

    fn wrap_text(splits: &[Vec<usize>], texts: &[Vec<&str>]) -> Vec<String> {
        let mut wrapped_strings = vec![];
        let mut already_checked = 0;

        for (text, split) in texts.iter().zip(splits) {
            // dbg!(&text, &split);
            for pos in split {
                let string = text[already_checked..*pos].join(" ");
                wrapped_strings.push(string);
                already_checked = *pos;
                // dbg!(string);
            }
            already_checked = 0;
        }

        wrapped_strings
    }
}

fn blank_buffer_new(w: u32, h: u32) -> RgbaImage {
    let mut image = RgbaImage::new((w as f32 * 1.2) as _, (h as f32 * 1.2) as _);
    for px in image.pixels_mut() {
        px.0 = [255, 255, 255, 255];
    }
    // println!("{:?}", image.to_vec());
    image
}
