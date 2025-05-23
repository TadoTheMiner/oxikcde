mod style;
use clap::ValueEnum;
use keybindings::parse_keybindings;
pub(super) use style::StylingConfig;
use style::StylingConfigRaw;

mod terminal;
pub(super) use terminal::TerminalConfig;
use terminal::TerminalConfigRaw;

mod keybindings;
use super::{Keybindings, SwitchToComic};
use bitflags::Flags;
use color_eyre::{Result, eyre::eyre, owo_colors::OwoColorize};
use figment::{
    Figment,
    providers::{Data, Toml},
};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};

const DEFAULT_CONFIG: &str = include_str!("../../default_config.toml");
pub fn print_default_config() {
    println!("{DEFAULT_CONFIG}")
}

pub fn write_default_config(path: &PathBuf) -> Result<()> {
    fs::write(path, DEFAULT_CONFIG)?;
    println!("Wrote default config to {}", path.display().green());
    Ok(())
}

pub(super) struct Config {
    pub keep_colors: bool,
    pub initial_comic: SwitchToComic,
    pub url: String,
    pub explanation_url: String,

    pub keybindings: Keybindings,
    pub styling: StylingConfig,
    pub terminal: TerminalConfig,
}

impl Config {
    pub fn new(path: &PathBuf) -> Result<Self> {
        let raw: ConfigRaw = Figment::new()
            .merge(Data::<Toml>::string(DEFAULT_CONFIG))
            .merge(Data::<Toml>::file(path))
            .extract()?;
        Ok(Self {
            keep_colors: raw.keep_colors,
            url: raw.url,
            initial_comic: SwitchToComic::from_str(&raw.initial_comic, false)
                .map_err(|e| eyre!("Failed to parse initial_comic config option: {e}"))?,
            explanation_url: raw.explanation_url,
            keybindings: parse_keybindings(raw.keybindings)?,
            styling: StylingConfig::from_raw(raw.styling)?,
            terminal: TerminalConfig::from_raw(raw.terminal)?,
        })
    }
}

#[derive(Deserialize)]
struct ConfigRaw {
    keep_colors: bool,
    initial_comic: String,
    url: String,
    explanation_url: String,

    styling: StylingConfigRaw,
    keybindings: HashMap<String, String>,
    terminal: TerminalConfigRaw,
}

type Modifiers<T, const L: usize> = [(&'static str, T); L];
fn parse_modifiers<T: Flags, const L: usize>(
    split: Vec<&str>,
    modifiers: Modifiers<T, L>,
) -> Result<T> {
    // TODO: this begs for optimization
    let modifier_strings: Vec<&&str> = modifiers.iter().map(|(str, _)| str).collect();
    if let Some(invalid_str) = split.iter().find(|str| !modifier_strings.contains(str)) {
        return Err(eyre!("Cannot parse {invalid_str}"));
    };
    Ok(modifiers
        .into_iter()
        .filter(|(str, _)| split.contains(str))
        .fold(T::empty(), |acc, (_, modifier)| acc.union(modifier)))
}
