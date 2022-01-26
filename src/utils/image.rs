use anyhow::Result;
use image::{GenericImage, GenericImageView, ImageBuffer, Pixel, Primitive, Rgba, RgbaImage};
use imageproc::drawing::{draw_text_mut, text_size};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rusttype::{Font, Scale};

use crate::error::ErrorKind;

/// Holds the basic requirements to create a caption image.
#[derive(Debug, Clone)]
pub struct SetUp {
    /// [`Font`] to be used.
    font: Font<'static>,
    /// [`Scale`] of the text.
    scale: Scale,
    /// Width of the input media.
    gif_w: u32,
}

impl SetUp {
    /// Initialize the setup to create a caption image.
    ///
    /// This function *must* be followed by the [`with_dimensions()`] functions.
    ///
    /// [`with_dimensions()`]: crate::utils::image::SetUp::with_dimensions()
    pub const fn init(font: Font<'static>) -> Self {
        Self {
            font,
            scale: Scale { x: 0.0, y: 0.0 },
            gif_w: 0,
        }
    }

    /// Adds the input media's dimensions to the struct.
    ///
    /// This function is *must* be used if contructing [`SetUp`]
    pub fn with_dimensions(self, width: u32, height: u32) -> Self {
        Self {
            gif_w: width,
            scale: Scale::uniform(height as f32 / 8.0),
            ..self
        }
    }

    /// Returns a reference to the [`Font`] of the image.
    pub const fn font(&self) -> &Font<'_> {
        &self.font
    }

    /// Returns the [`Scale`] of the text.
    pub const fn scale(&self) -> Scale {
        self.scale
    }
}

/// Text Image is the second building block of an image caption.
#[derive(Debug, Clone)]
pub struct TextImage {
    init: SetUp,
    text: Vec<String>,
}

impl TextImage {
    /// Create a new [`TextImage`] to be used to image captioning.
    pub fn new(init: SetUp, text: &str) -> Self {
        let text_nlsplit: Vec<_> = text.split('\n').collect();
        let split_texts: Vec<Vec<_>> = text_nlsplit
            .iter()
            .map(|str| str.split_whitespace().collect())
            .collect();

        let scale = init.scale();

        let text = Self::wrap_text(
            &Self::sum_until_fit(scale, init.font(), init.gif_w as i32, &split_texts),
            &split_texts,
        );

        Self { init, text }
    }

    /// Render a [`TextImage`] into an image caption.
    ///
    /// Returns a correctly scaled [`ImageBuffer`] of the caption.
    ///
    /// # Errors
    /// This function will return an error if the caption text
    /// spans multiple lines and cannot be concatenated into
    /// a single image.
    /// See also: [`v_concat()`]
    ///
    /// [`v_concat()`]: crate::utils::image::TextImage::v_concat()
    pub fn render(self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        let single = self.text.len() == 1;
        let height = self.max_height()?;
        let image = if single {
            // this is fine because there is only one element
            // and so we do not need to concatenate images.
            self.render_text(&self.text[0], height, single)
        } else {
            let images: Vec<_> = self
                .text
                .par_iter()
                .map(|text| {
                    self.render_text(text, height, single)
                })
                .collect();
            Self::v_concat(&images)?
        };

        let image_h = image.height();
        let image = Self::set_bg(&image, self.init.gif_w);
        Ok(Self::resize(&image, self.init.gif_w, image_h as _))
    }

    /// Renders a single line of text.
    ///
    /// Returns a transparent [`ImageBuffer`] with one line of caption
    /// drawn.  
    fn render_text(&self, text: &str, height: i32, single: bool) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (text_width, text_height) = text_size(self.init.scale(), self.init.font(), text);
        // padding for the text up and down
        let height = (height as f32 * if single { 2.5 } else { 1.3 }) as u32;
        let mut image = RgbaImage::new(text_width as u32, height as u32);
        let y_offset = (image.height() as i32 - text_height) / 2;
        draw_text_mut(
            &mut image,
            Rgba([0_u8, 0_u8, 0_u8, 255_u8]),
            0,
            y_offset,
            self.init.scale(),
            self.init.font(),
            text,
        );
        image
    }

    /// Returns the maximum height of the rendered text.
    fn max_height(&self) -> Result<i32> {
        let dimensions = |txt| text_size(self.init.scale(), self.init.font(), txt);
        let h: Vec<_> = self.text.iter().map(|t| dimensions(t).1).collect();
        let max_height = match h.iter().max_by_key(|&&x| x) {
            Some(val) => *val,
            None => return Err(ErrorKind::NoTextGiven.into()),
        };

        Ok(max_height)
    }

    /// Resizes the supplied image to a given width.
    ///
    /// Captions cannot exceed the width of the input media, which is why
    /// the caption image needs to be resized to fit the width accordingly.
    ///
    /// This operation preserves aspect ratio.
    fn resize(
        image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        t_width: u32,
        image_h: u32,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        // rounding up because ffmpeg doesnt
        // play well with non-even numbers in resolutions
        let image_h = if image_h % 2 != 0 {
            image_h + 1
        } else {
            image_h
        };
        image::imageops::resize(
            image,
            Self::npercent(image.width(), t_width),
            image_h as _,
            image::imageops::FilterType::Gaussian,
        )
    }

    /// Overlays the text image on white buffer.
    ///
    /// This caption text image is centered.
    fn set_bg(
        buffer: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        gif_w: u32,
    ) -> image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>> {
        let mut bg = blank_buffer_new(gif_w, buffer.height() as _);

        let (x, y) = {
            let (bg_h, bg_w) = (bg.height() as i32, bg.width() as i32);
            let (img_h, img_w) = (buffer.height() as i32, buffer.width() as i32);
            ((bg_w - img_w) / 2, (bg_h - img_h) / 2)
        };

        image::imageops::overlay(&mut bg, buffer, x as _, y as _);
        bg
    }

    /// Concatenates a collection of images vertically.
    ///
    /// This allows the program to draw individual lines at a time
    /// and stitch them together vertically.
    pub fn v_concat<I, P, S>(images: &[I]) -> Result<ImageBuffer<P, Vec<S>>>
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
            imgbuf.copy_from(img, (img_width_out - img.width()) / 2, accumulated_height)?;
            accumulated_height += img.height();
        }

        Ok(imgbuf)
    }

    /// Wraps text in accodance to the input image width.
    ///
    /// Returns a `Vec<Vec<usize>>` with indexes of where the splits should occur.
    /// This function can handle manual newlines when they are in separate vectors:
    ///
    /// ```no_run
    /// let text = "insert here\nsomething generic";
    /// ```
    /// Split and collect at newline and words:
    /// ```no_run
    /// let split_texts: Vec<Vec<_>> = text
    ///     .iter()
    ///     .map(|str| str.split_whitespace().collect())
    ///     .collect();
    /// ```
    /// This is now looks like this:
    /// ```no_run
    /// let text = vec![
    ///     [  "insert", "here" ], // these two internal vectors are seperated due to a newline
    ///     [  "something", "generic" ]
    /// ];
    /// ```
    /// Now, `text` can be passed into the function to create indices where splits should occur.
    pub fn sum_until_fit(
        scale: Scale,
        font: &Font,
        image_width: i32,
        split_texts: &[Vec<&str>],
    ) -> Vec<Vec<usize>> {
        let mut split_at = vec![];
        for elem in split_texts {
            let mut accumulator = 0;
            let str_widths: Vec<i32> = elem
                .iter()
                .map(|text| text_size(scale, font, text).0)
                .collect();

            let mut tempvec = Vec::with_capacity(str_widths.len() + 1);
            for x in 0..str_widths.len() {
                if str_widths[accumulator..x + 1].iter().sum::<i32>() > image_width {
                    tempvec.push(x);
                    accumulator = x;
                }
            }

            tempvec.push(str_widths.len());
            split_at.push(tempvec);
        }
        split_at
    }

    /// Helper function to return the npercent of the text to be resized.
    fn npercent(current_width: u32, target_width: u32) -> u32 {
        (current_width as f32 * (target_width as f32 / current_width as f32)) as u32
    }

    /// Splits the given texts into multiple [`String`]s.
    ///
    /// These strings are split according to the given splits.
    /// See also: [`sum_until_fit()`]
    ///
    /// [`sum_until_fit()`]: crate::utils::image::TextImage::sum_until_fit()
    pub fn wrap_text(splits: &[Vec<usize>], texts: &[Vec<&str>]) -> Vec<String> {
        let mut wrapped_strings = vec![];
        let mut already_checked = 0;

        for (text, split) in texts.iter().zip(splits) {
            for pos in split {
                let string = text[already_checked..*pos].join(" ");
                wrapped_strings.push(string);
                already_checked = *pos;
            }
            already_checked = 0;
        }
        wrapped_strings
    }
}

/// Create a new white image buffer.
fn blank_buffer_new(w: u32, h: u32) -> RgbaImage {
    // text with only one word on it, does not have enough padding to look good
    let scale_factor: f32 = 1.2;
    let mut image = RgbaImage::new(
        (w as f32 * scale_factor) as _,
        (h as f32 * scale_factor) as _,
    );

    for px in image.pixels_mut() {
        px.0 = [255, 255, 255, 255];
    }
    image
}
