use harper_core::linting::LintGroup;
use harper_core::{Document, FullDictionary, Lrc, MergedDictionary};

#[derive(Default)]
pub struct DocumentState {
    pub document: Document,
    pub ident_dict: Lrc<FullDictionary>,
    pub dict: Lrc<MergedDictionary<FullDictionary>>,
    pub linter: LintGroup<Lrc<MergedDictionary<FullDictionary>>>,
    pub language_id: Option<String>
}
