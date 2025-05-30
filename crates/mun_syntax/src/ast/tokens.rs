//! There are many `AstNodes`, but only a few tokens, so we hand-write them
//! here.

use crate::{
    ast::AstToken,
    SyntaxKind,
    SyntaxKind::{COMMENT, WHITESPACE},
    SyntaxToken,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comment(SyntaxToken);

impl AstToken for Comment {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == COMMENT
    }
    fn cast(token: SyntaxToken) -> Option<Self> {
        if Self::can_cast(token.kind()) {
            Some(Comment(token))
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
}

impl Comment {
    pub fn kind(&self) -> CommentKind {
        kind_by_prefix(self.text())
    }

    pub fn prefix(&self) -> &'static str {
        prefix_by_kind(self.kind())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CommentKind {
    pub shape: CommentShape,
    pub doc: Option<CommentPlacement>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CommentShape {
    Line,
    Block,
}

impl CommentShape {
    pub fn is_line(self) -> bool {
        self == CommentShape::Line
    }

    pub fn is_block(self) -> bool {
        self == CommentShape::Block
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CommentPlacement {
    Inner,
    Outer,
}

const COMMENT_PREFIX_TO_KIND: &[(&str, CommentKind)] = {
    use CommentPlacement::{Inner, Outer};
    use CommentShape::{Block, Line};
    &[
        (
            "///",
            CommentKind {
                shape: Line,
                doc: Some(Outer),
            },
        ),
        (
            "//!",
            CommentKind {
                shape: Line,
                doc: Some(Inner),
            },
        ),
        (
            "/**",
            CommentKind {
                shape: Block,
                doc: Some(Outer),
            },
        ),
        (
            "/*!",
            CommentKind {
                shape: Block,
                doc: Some(Inner),
            },
        ),
        (
            "//",
            CommentKind {
                shape: Line,
                doc: None,
            },
        ),
        (
            "/*",
            CommentKind {
                shape: Block,
                doc: None,
            },
        ),
    ]
};

fn kind_by_prefix(text: &str) -> CommentKind {
    for (prefix, kind) in COMMENT_PREFIX_TO_KIND.iter() {
        if text.starts_with(prefix) {
            return *kind;
        }
    }
    panic!("bad comment text: {text:?}")
}

fn prefix_by_kind(kind: CommentKind) -> &'static str {
    for (prefix, k) in COMMENT_PREFIX_TO_KIND.iter() {
        if *k == kind {
            return prefix;
        }
    }
    unreachable!()
}

pub struct Whitespace(SyntaxToken);

impl AstToken for Whitespace {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == WHITESPACE
    }
    fn cast(token: SyntaxToken) -> Option<Self> {
        if Self::can_cast(token.kind()) {
            Some(Whitespace(token))
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
}

impl Whitespace {
    pub fn spans_multiple_lines(&self) -> bool {
        let text = self.text();
        text.find('\n')
            .is_some_and(|idx| text[idx + 1..].contains('\n'))
    }
}
