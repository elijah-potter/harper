use harper_core::Document;
use harper_data::Lrc;
use harper_linting::LintGroup;
use harper_spell::MergedDictionary;

#[derive(Default)]
pub struct DocumentState {
    pub document: Document,
    pub dict: Lrc<MergedDictionary>,
    pub linter: LintGroup<Lrc<MergedDictionary>>,
    pub language_id: Option<String>,
}
