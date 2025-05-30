use std::{
    path::{Component, Path, Prefix},
    str::FromStr,
};

use lsp_types::Url;
use mun_hir_input::{FileId, LineIndex};
use mun_syntax::{TextRange, TextSize};

use crate::{
    completion::{CompletionItem, CompletionItemKind},
    state::LanguageServerSnapshot,
    symbol_kind::SymbolKind,
};

/// Returns a `Url` object from a given path, will lowercase drive letters if
/// present. This will only happen when processing Windows paths.
///
/// When processing non-windows path, this is essentially do the same as
/// `Url::from_file_path`.
fn url_from_path_with_drive_lowercasing(path: impl AsRef<Path>) -> anyhow::Result<Url> {
    let component_has_windows_drive = path.as_ref().components().any(|comp| {
        if let Component::Prefix(c) = comp {
            match c.kind() {
                Prefix::Disk(_) | Prefix::VerbatimDisk(_) => return true,
                _ => return false,
            }
        }
        false
    });

    // VSCode expects drive letters to be lowercased, whereas rust will uppercase
    // the drive letters.
    if component_has_windows_drive {
        let url_original = Url::from_file_path(&path).map_err(|()| {
            anyhow::anyhow!("can't convert path to url: {}", path.as_ref().display())
        })?;

        let drive_partition: Vec<&str> = url_original.as_str().rsplitn(2, ':').collect();

        // There is a drive partition, but we never found a colon.
        // This should not happen, but in this case we just pass it through.
        if drive_partition.len() == 1 {
            return Ok(url_original);
        }

        let joined = drive_partition[1].to_ascii_lowercase() + ":" + drive_partition[0];
        let url = Url::from_str(&joined).expect("This came from a valid `Url`");

        Ok(url)
    } else {
        Ok(Url::from_file_path(&path).map_err(|()| {
            anyhow::anyhow!("can't convert path to url: {}", path.as_ref().display())
        })?)
    }
}

pub(crate) fn range(range: TextRange, line_index: &LineIndex) -> lsp_types::Range {
    lsp_types::Range {
        start: position(range.start(), line_index),
        end: position(range.end(), line_index),
    }
}

pub(crate) fn position(range: TextSize, line_index: &LineIndex) -> lsp_types::Position {
    let line_col = line_index.line_col(range);
    lsp_types::Position {
        line: line_col.line,
        character: line_col.col_utf16,
    }
}

/// Converts a symbol kind from this crate to one for the LSP protocol.
pub(crate) fn symbol_kind(symbol_kind: SymbolKind) -> lsp_types::SymbolKind {
    match symbol_kind {
        SymbolKind::Function => lsp_types::SymbolKind::FUNCTION,
        SymbolKind::Struct => lsp_types::SymbolKind::STRUCT,
        SymbolKind::TypeAlias | SymbolKind::SelfType => lsp_types::SymbolKind::TYPE_PARAMETER,
        SymbolKind::Field => lsp_types::SymbolKind::FIELD,
        SymbolKind::Local | SymbolKind::SelfParam => lsp_types::SymbolKind::VARIABLE,
        SymbolKind::Module => lsp_types::SymbolKind::MODULE,
        SymbolKind::Method => lsp_types::SymbolKind::METHOD,
        SymbolKind::Impl => lsp_types::SymbolKind::OBJECT,
    }
}

/// Returns the `Url` associated with the specified `FileId`.
pub(crate) fn url(snapshot: &LanguageServerSnapshot, file_id: FileId) -> anyhow::Result<Url> {
    let vfs = snapshot.vfs.read();
    let path = vfs.file_path(mun_vfs::FileId(file_id.0));
    let url = url_from_path_with_drive_lowercasing(path)?;
    Ok(url)
}

/// Converts from a list of our `CompletionItem` to an LSP `CompletionItem`
pub(crate) fn completion_items(
    completion_items: Vec<CompletionItem>,
) -> Vec<lsp_types::CompletionItem> {
    let max_relevance = completion_items
        .iter()
        .map(|it| it.relevance.score())
        .max()
        .unwrap_or_default();
    completion_items
        .into_iter()
        .map(|it| completion_item(max_relevance, it))
        .collect()
}

/// Converts from our `CompletionItem` to an LSP `CompletionItem`
pub(crate) fn completion_item(
    max_relevance: u32,
    completion_item: CompletionItem,
) -> lsp_types::CompletionItem {
    // Compute the score_text based on the relevance of the completion item
    let relevance = completion_item.relevance;
    let preselect = if relevance.is_relevant() && relevance.score() == max_relevance {
        Some(true)
    } else {
        None
    };

    // The relevance needs to be inverted to come up with a sort score
    // because the client will sort ascending.
    let sort_score = relevance.score() ^ 0xFF_FF_FF_FF;

    // Zero pad the string to ensure values can be properly sorted
    // by the client. Hex format is used because it is easier to
    // visually compare very large values, which the sort text
    // tends to be since it is the opposite of the score.
    let sort_text = Some(format!("{sort_score:08x}"));

    lsp_types::CompletionItem {
        label: completion_item.label,
        kind: Some(completion_item_kind(completion_item.kind)),
        detail: completion_item.detail,
        preselect,
        sort_text,
        ..Default::default()
    }
}

pub(crate) fn completion_item_kind(
    completion_item_kind: CompletionItemKind,
) -> lsp_types::CompletionItemKind {
    match completion_item_kind {
        CompletionItemKind::Binding => lsp_types::CompletionItemKind::VARIABLE,
        CompletionItemKind::BuiltinType => lsp_types::CompletionItemKind::STRUCT,
        CompletionItemKind::Keyword => lsp_types::CompletionItemKind::KEYWORD,
        CompletionItemKind::Method => lsp_types::CompletionItemKind::METHOD,
        CompletionItemKind::Snippet => lsp_types::CompletionItemKind::SNIPPET,
        CompletionItemKind::UnresolvedReference => lsp_types::CompletionItemKind::REFERENCE,
        CompletionItemKind::SymbolKind(symbol) => match symbol {
            SymbolKind::Field => lsp_types::CompletionItemKind::FIELD,
            SymbolKind::Function => lsp_types::CompletionItemKind::FUNCTION,
            SymbolKind::Local => lsp_types::CompletionItemKind::VARIABLE,
            SymbolKind::Module => lsp_types::CompletionItemKind::MODULE,
            SymbolKind::SelfParam => lsp_types::CompletionItemKind::VALUE,
            SymbolKind::SelfType => lsp_types::CompletionItemKind::TYPE_PARAMETER,
            SymbolKind::Struct | SymbolKind::TypeAlias => lsp_types::CompletionItemKind::STRUCT,
            SymbolKind::Method => lsp_types::CompletionItemKind::METHOD,
            SymbolKind::Impl => lsp_types::CompletionItemKind::TEXT,
        },
        CompletionItemKind::Attribute => lsp_types::CompletionItemKind::ENUM_MEMBER,
    }
}
