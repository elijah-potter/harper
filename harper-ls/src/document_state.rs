use harper_core::linting::LintGroup;
use harper_core::{Document, FullDictionary, MergedDictionary};
use harper_data::Lrc;

#[derive(Default)]
pub struct DocumentState {
    pub document: Document,
    pub ident_dict: Lrc<FullDictionary>,
    pub dict: Lrc<MergedDictionary>,
    pub linter: LintGroup<Lrc<MergedDictionary>>,
    pub language_id: Option<String>,
}
