use std::path::Path;

use crate::{comment_parsers, tree_sitter_masker::TreeSitterMasker};
use comment_parsers::{Go, JsDoc, Unit};
use harper_core::{
    parsers::{self, Parser},
    FullDictionary, Token,
};

pub struct TreeSitterParser {
    inner: parsers::Mask<TreeSitterMasker, Box<dyn Parser>>,
}

impl TreeSitterParser {
    pub fn create_ident_dict(&self, source: &[char]) -> Option<FullDictionary> {
        self.inner.masker.create_ident_dict(source)
    }

    pub fn new_from_language_id(language_id: &str) -> Option<Self> {
        let language = match language_id {
            "rust" => tree_sitter_rust::language(),
            "typescriptreact" => tree_sitter_typescript::language_tsx(),
            "typescript" => tree_sitter_typescript::language_typescript(),
            "py" => tree_sitter_python::language(),
            "javascript" => tree_sitter_javascript::language(),
            "javascriptreact" => tree_sitter_typescript::language_tsx(),
            "go" => tree_sitter_go::language(),
            "c" => tree_sitter_c::language(),
            "cpp" => tree_sitter_cpp::language(),
            "ruby" => tree_sitter_ruby::language(),
            "swift" => tree_sitter_swift::language(),
            "csharp" => tree_sitter_c_sharp::language(),
            "toml" => tree_sitter_toml::language(),
            "lua" => tree_sitter_lua::language(),
            "sh" => tree_sitter_bash::language(),
            "java" => tree_sitter_java::language(),
            _ => return None,
        };

        let comment_parser: Box<dyn Parser> = match language_id {
            "javascriptreact" | "typescript" | "typescriptreact" | "javascript" => Box::new(JsDoc),
            "go" => Box::new(Go),
            _ => Box::new(Unit),
        };

        Some(Self {
            inner: parsers::Mask::new(TreeSitterMasker::new(language), comment_parser),
        })
    }

    /// Infer the programming language from a provided filename.
    pub fn new_from_filename(filename: &Path) -> Option<Self> {
        Self::new_from_language_id(Self::filename_to_filetype(filename)?)
    }

    /// Convert a provided path to a corresponding Language Server Protocol file
    /// type.
    ///
    /// Note to contributors: try to keep this in sync with
    /// [`Self::new_from_language_id`]
    fn filename_to_filetype(path: &Path) -> Option<&'static str> {
        Some(match path.extension()?.to_str()? {
            "rs" => "rust",
            "ts" => "typescript",
            "tsx" => "typescriptreact",
            "js" => "javascript",
            "jsx" => "javascriptreact",
            "go" => "go",
            "c" => "c",
            "cpp" => "cpp",
            "h" => "cpp",
            "rb" => "ruby",
            "swift" => "swift",
            "cs" => "csharp",
            "toml" => "toml",
            "lua" => "lua",
            "sh" => "sh",
            "bash" => "sh",
            "java" => "java",
            _ => return None,
        })
    }
}

impl Parser for TreeSitterParser {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        self.inner.parse(source)
    }
}
