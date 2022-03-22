use anyhow::Result;
use image::{GenericImage, GenericImageView, ImageBuffer, Pixel, Primitive, Rgba};
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
        let text = text.wrap(&init);
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
                .map(|text| self.render_text(text, height, single))
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
        let mut image = ImageBuffer::new(text_width as u32, height as u32);
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
        let image_h = if image_h % 2 == 0 {
            image_h
        } else {
            image_h + 1
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
    ) -> image::ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut bg = blank_buffer_new(gif_w, buffer.height() as _);

        let (x, y) = {
            let (bg_h, bg_w) = (bg.height() as i32, bg.width() as i32);
            let (img_h, img_w) = (buffer.height() as i32, buffer.width() as i32);
            ((bg_w - img_w) / 2, (bg_h - img_h) / 2)
        };

        image::imageops::overlay(&mut bg, buffer, x.into(), y.into());
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

    /// Helper function to return the npercent of the text to be resized.
    fn npercent(current_width: u32, target_width: u32) -> u32 {
        (current_width as f32 * (target_width as f32 / current_width as f32)) as u32
    }
}

/// Create a new white image buffer. The returned [`ImageBuffer`] will
/// have a size 1.2 times the given width and height.
fn blank_buffer_new(w: u32, h: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut image: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::new((w as f32 * 1.2) as u32, (h as f32 * 1.2) as u32);
    for px in image.pixels_mut() {
        px.0 = [255, 255, 255, 255];
    }
    image
}

/// Implements text wrap with the greedy algorithm.
trait Wrap {
    /// Wraps text.
    fn wrap(&self, setup: &SetUp) -> Vec<String>;
}

impl Wrap for &str {
    fn wrap(&self, setup: &SetUp) -> Vec<String> {
        let widthcalc = |text: &str| text_size(setup.scale, &setup.font, text).0;
        let letter_width = widthcalc("W");
        let mut space_left = setup.gif_w as i32;
        let mut line = String::new();
        for sentence in self.split('\n') {
            for word in sentence.split_whitespace() {
                if widthcalc(word) + letter_width > space_left {
                    line.push('\n'); // line break char
                    space_left = setup.gif_w as i32 - widthcalc(word);
                } else {
                    space_left -= widthcalc(word) + letter_width;
                }
                let s = word.trim().to_owned() + " ";
                line.push_str(&s);
            }
        }

        line.split('\n').map(ToOwned::to_owned).collect()
    }
}
