use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    WebLanguageRequester,
};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n"]
struct Localizations;

pub static STATIC_LOADER: Lazy<FluentLanguageLoader> = Lazy::new(|| {
    let loader = fluent_language_loader!();
    let requested_languages = WebLanguageRequester::requested_languages();
    i18n_embed::select(&loader, &Localizations, &requested_languages).unwrap();
    loader
});

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::STATIC_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::STATIC_LOADER, $message_id, $($args), *)
    }};
}
