#![forbid(unsafe_code)]

mod messages;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use icu_locale_core::Locale;
pub use messages::*;
use serde::{Deserialize, Serialize};

pub const REGISTRY: &str = include_str!("../locales.json");

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppLocale {
    #[default]
    #[serde(rename = "en")]
    En,
    #[serde(rename = "zh-Hans")]
    ZhHans,
    #[serde(rename = "ja")]
    Ja,
    #[serde(rename = "ru")]
    Ru,
}
impl AppLocale {
    pub const fn tag(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::ZhHans => "zh-Hans",
            Self::Ja => "ja",
            Self::Ru => "ru",
        }
    }
    pub fn negotiate<'a>(candidates: impl IntoIterator<Item = &'a str>) -> Self {
        candidates
            .into_iter()
            .find_map(Self::matching)
            .unwrap_or_default()
    }
    pub fn matching(candidate: &str) -> Option<Self> {
        let locale: Locale = candidate.parse().ok()?;
        let tag = locale.id;
        match tag.language.as_str() {
            "en" => Some(Self::En),
            "ja" => Some(Self::Ja),
            "ru" => Some(Self::Ru),
            "zh" => {
                if let Some(script) = tag.script {
                    return (script.as_str() == "Hans").then_some(Self::ZhHans);
                }
                match tag.region.as_ref().map(|r| r.as_str()) {
                    None | Some("CN" | "SG") => Some(Self::ZhHans),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Domain {
    Game,
    Report,
    Cli,
    Errors,
    Diagnostics,
}
impl Domain {
    fn source(self, locale: AppLocale) -> &'static str {
        match (self, locale) {
            (Self::Game, AppLocale::En) => include_str!("../locales/en/game.ftl"),
            (Self::Game, AppLocale::Ja) => include_str!("../locales/ja/game.ftl"),
            (Self::Game, AppLocale::Ru) => include_str!("../locales/ru/game.ftl"),
            (Self::Game, AppLocale::ZhHans) => include_str!("../locales/zh-Hans/game.ftl"),
            (Self::Report, AppLocale::En) => include_str!("../locales/en/report.ftl"),
            (Self::Report, AppLocale::Ja) => include_str!("../locales/ja/report.ftl"),
            (Self::Report, AppLocale::Ru) => include_str!("../locales/ru/report.ftl"),
            (Self::Report, AppLocale::ZhHans) => include_str!("../locales/zh-Hans/report.ftl"),
            (Self::Cli, AppLocale::En) => include_str!("../locales/en/cli.ftl"),
            (Self::Cli, AppLocale::Ja) => include_str!("../locales/ja/cli.ftl"),
            (Self::Cli, AppLocale::Ru) => include_str!("../locales/ru/cli.ftl"),
            (Self::Cli, AppLocale::ZhHans) => include_str!("../locales/zh-Hans/cli.ftl"),
            (Self::Errors, AppLocale::En) => include_str!("../locales/en/errors.ftl"),
            (Self::Errors, AppLocale::Ja) => include_str!("../locales/ja/errors.ftl"),
            (Self::Errors, AppLocale::Ru) => include_str!("../locales/ru/errors.ftl"),
            (Self::Errors, AppLocale::ZhHans) => include_str!("../locales/zh-Hans/errors.ftl"),
            (Self::Diagnostics, AppLocale::En) => include_str!("../locales/en/diagnostics.ftl"),
            (Self::Diagnostics, AppLocale::Ja) => include_str!("../locales/ja/diagnostics.ftl"),
            (Self::Diagnostics, AppLocale::Ru) => include_str!("../locales/ru/diagnostics.ftl"),
            (Self::Diagnostics, AppLocale::ZhHans) => {
                include_str!("../locales/zh-Hans/diagnostics.ftl")
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("translation resource error: {0}")]
pub struct TranslationError(pub String);

/// Immutable task-local presentation context; the game domain must never emit bidi controls.
pub struct Translator {
    primary: FluentBundle<FluentResource>,
    english: FluentBundle<FluentResource>,
}
impl Translator {
    pub fn new(locale: AppLocale, domain: Domain) -> Result<Self, TranslationError> {
        fn bundle(
            locale: AppLocale,
            domain: Domain,
        ) -> Result<FluentBundle<FluentResource>, TranslationError> {
            let language = locale
                .tag()
                .parse()
                .map_err(|e| TranslationError(format!("{e:?}")))?;
            let resource = FluentResource::try_new(domain.source(locale).to_owned())
                .map_err(|(_, errors)| TranslationError(format!("{errors:?}")))?;
            let mut bundle = FluentBundle::new(vec![language]);
            bundle.set_use_isolating(!matches!(domain, Domain::Game));
            bundle
                .add_resource(resource)
                .map_err(|errors| TranslationError(format!("{errors:?}")))?;
            Ok(bundle)
        }
        Ok(Self {
            primary: bundle(locale, domain)?,
            english: bundle(AppLocale::En, domain)?,
        })
    }
    fn text(&self, id: &str, args: Option<&FluentArgs<'_>>) -> Result<String, TranslationError> {
        for bundle in [&self.primary, &self.english] {
            if let Some(pattern) = bundle.get_message(id).and_then(|m| m.value()) {
                let mut errors = Vec::new();
                let value = bundle.format_pattern(pattern, args, &mut errors);
                if errors.is_empty() {
                    return Ok(value.into_owned());
                }
            }
        }
        Err(TranslationError(id.to_owned()))
    }
    pub fn game_title(&self) -> Result<String, TranslationError> {
        self.text("game-title", None)
    }
    pub fn game_max_level(&self) -> Result<String, TranslationError> {
        self.text("game-max-level", None)
    }
    pub fn game_tier(&self, tier: i32) -> Result<String, TranslationError> {
        let mut args = FluentArgs::new();
        args.set("tier", tier);
        self.text("game-tier", Some(&args))
    }
    pub fn game_requires(&self, items: &str) -> Result<String, TranslationError> {
        let mut args = FluentArgs::new();
        args.set("items", items);
        self.text("game-requires", Some(&args))
    }
    pub fn game_or(&self, left: &str, right: &str) -> Result<String, TranslationError> {
        let mut args = FluentArgs::new();
        args.set("left", left);
        args.set("right", right);
        self.text("game-or", Some(&args))
    }
    pub fn tree_omitted(
        &self,
        root: &str,
        count: usize,
        limit: usize,
    ) -> Result<String, TranslationError> {
        let mut args = FluentArgs::new();
        args.set("root", root);
        args.set("count", count);
        args.set("limit", limit);
        self.text("game-tree-omitted", Some(&args))
    }
}
