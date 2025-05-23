mod image;
pub mod terminal;

use super::{comic::Comic, config::StylingConfig, config::TerminalConfig};
use ::image::DynamicImage;
use color_eyre::Result;
use image::*;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::Styled,
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};
use ratatui_image::{Resize, protocol::StatefulProtocol};
use terminal::*;

pub struct Ui {
    terminal: DefaultTerminal,
    image_protocols: Option<ImageProtocols>,
    image_processor: ImageProcessor,
    styling_config: StylingConfig,
    message: Option<Span<'static>>,
}

pub enum RenderOption {
    ShowError(String),
    ShowMessage(&'static str),
    NewImage(DynamicImage),
    DeleteMessage,
    None,
}

impl Ui {
    pub fn new(
        styling_config: StylingConfig,
        terminal_config: TerminalConfig,
        keep_colors: bool,
    ) -> Result<Self> {
        let terminal = initialise_terminal()?;
        let image_processor = ImageProcessor::new(
            terminal_config
                .foreground_color
                .map(Ok)
                .unwrap_or_else(|| get_color(FOREGROUND_COLOR))?,
            terminal_config
                .background_color
                .map(Ok)
                .unwrap_or_else(|| get_color(BACKGROUND_COLOR))?,
            keep_colors,
        )?;
        Ok(Self {
            terminal,
            styling_config,
            image_protocols: None,
            image_processor,
            message: None,
        })
    }

    pub fn update(
        &mut self,
        comic: &Comic,
        process_image: bool,
        option: RenderOption,
    ) -> Result<()> {
        let current_message = self.message.take();
        self.message = match option {
            RenderOption::ShowMessage(message) => {
                Some(message.set_style(self.styling_config.messages_style))
            }
            RenderOption::ShowError(error) => {
                Some(error.set_style(self.styling_config.errors_style))
            }
            RenderOption::NewImage(image) => {
                self.image_protocols = Some(self.image_processor.image_protocols(image));
                None
            }
            RenderOption::None => current_message,
            RenderOption::DeleteMessage => {
                if current_message.is_none() {
                    return Ok(());
                }
                None
            }
        };

        let title_block = Block::new()
            .title_top(
                comic
                    .date_uploaded()
                    .set_style(self.styling_config.date_style),
            )
            .title_top(
                Line::styled(format!("{comic}"), self.styling_config.title_style).centered(),
            );

        let title_block = if let Some(message) = self.message.clone() {
            title_block.title_top(message.into_right_aligned_line())
        } else {
            title_block
        };

        let alt_text = Paragraph::new(comic.alt_text())
            .centered()
            .wrap(Wrap::default())
            .set_style(self.styling_config.alt_text_style);

        self.terminal.draw(|frame| {
            render(
                title_block,
                alt_text,
                self.image_protocols
                    .as_mut()
                    .map(|protocols| protocols.get(process_image)),
                frame,
            )
        })?;
        Ok(())
    }

    pub fn clear_image_protocols(&mut self) {
        self.image_protocols = None;
    }
}

fn render(
    title_block: Block,
    alt_text: Paragraph,
    image: Option<&mut StatefulProtocol>,
    frame: &mut Frame,
) {
    let alt_text_height = alt_text.line_count(frame.area().width) as u16;
    let layout = layout(alt_text_height).split(frame.area());
    frame.render_widget(title_block, layout[0]);
    frame.render_widget(alt_text, layout[2]);
    if let Some(image) = image {
        let image_area = image.size_for(&Resize::Scale(None), layout[1]);
        let centered_image_area = center_area(
            layout[1],
            Constraint::Length(image_area.width),
            Constraint::Length(image_area.height),
        );

        frame.render_stateful_widget(IMAGE_WIDGET, centered_image_area, image)
    };
}

fn center_area(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

fn layout(alt_text_height: u16) -> Layout {
    Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(alt_text_height),
        ],
    )
}
