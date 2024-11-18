use color_eyre::Result;
use image::{
    imageops::{grayscale, invert},
    DynamicImage,
};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Wrap},
    DefaultTerminal,
};
use ratatui_image::{picker::Picker, Image, Resize};

use super::comic::Comic;
pub struct Ui {
    terminal: DefaultTerminal,
    picker: Picker,
    invert_image: bool,
    comic: Comic,
}

impl Ui {
    pub fn new(terminal: DefaultTerminal, comic: Comic) -> Result<Self> {
        let picker = Picker::from_query_stdio()?;
        let mut ui = Self {
            terminal,
            picker,
            invert_image: true,
            comic,
        };
        ui.render()?;
        Ok(ui)
    }
    pub fn handle_resize(&mut self) -> Result<()> {
        self.picker = Picker::from_query_stdio()?;
        self.render()
    }

    pub fn render_new_comic(&mut self, comic: Comic) -> Result<()> {
        self.comic = comic;
        self.invert_image = true;
        self.render()
    }

    pub fn toggle_invert(&mut self) -> Result<()> {
        self.invert_image = !self.invert_image;
        self.render()
    }

    fn render(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            let area = f.area();
            f.render_widget(
                Block::new()
                    .title_top(self.comic.date_uploaded.as_str().blue())
                    .title_top(
                        Line::styled(
                            format!("{}: {}", self.comic.number, self.comic.name),
                            Style::new().yellow().bold(),
                        )
                        .centered(),
                    ),
                area,
            );
            let alt_text = Paragraph::new(self.comic.alt_text.as_str())
                .centered()
                .wrap(Wrap::default())
                .dark_gray();
            let alt_text_height = alt_text.line_count(area.width) as u16;
            let alt_text_area = Rect {
                y: area.height - alt_text_height,
                height: alt_text_height,
                ..area
            };
            f.render_widget(alt_text, alt_text_area);
            let image_area = Rect {
                y: area.y + 1,
                height: area.height - 1 - alt_text_height,
                ..area
            };

            let image = self
                .picker
                .new_protocol(
                    if self.invert_image {
                        invert_image(&self.comic.image)
                    } else {
                        self.comic.image.clone()
                    },
                    image_area,
                    Resize::Fit(None),
                )
                .expect("XKCD should always contain valid images");

            // The image widget.
            //TODO: resize the image
            let image_widget = Image::new(&image);
            f.render_widget(image_widget, image_area)
        })?;
        Ok(())
    }
}

fn invert_image(image: &DynamicImage) -> DynamicImage {
    let mut grayscale = grayscale(image);
    invert(&mut grayscale);
    grayscale.into()
}
