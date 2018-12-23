use std::hash::{Hash, Hasher};

use syntax::ast::Expr;
use syntax_pos::Span;
use syntax::ext::base::{ExtCtxt, MacEager, MacResult};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;
use syntax::symbol::LocalInternedString;
use rustc_data_structures::sync::Lrc;

use phf_generator::HashState;

#[derive(PartialEq, Eq, Clone)]
pub enum Key {
    Str(LocalInternedString),
    Binary(Lrc<Vec<u8>>),
}

impl Hash for Key {
    fn hash<S: Hasher>(&self, state: &mut S) {
        match *self {
            Key::Str(ref s) => s.hash(state),
            Key::Binary(ref b) => b.hash(state),
        }
    }
}

impl AsRef<[u8]> for Key {
    fn as_ref(&self) -> &[u8] {
        match *self {
            Key::Str(ref s) => s.as_ref(),
            Key::Binary(ref s) => s.as_ref(),
        }
    }
}

pub struct Entry {
    pub key_contents: Key,
    pub key: P<Expr>,
    pub value: P<Expr>,
}

impl AsRef<[u8]> for Entry {
    fn as_ref(&self) -> &[u8] {
        self.key_contents.as_ref()
    }
}

pub fn create_map(
    cx: &mut ExtCtxt,
    sp: Span,
    entries: Vec<Entry>,
    state: HashState,
) -> Box<MacResult + 'static> {
    let disps = state
        .disps
        .iter()
        .map(|&(d1, d2)| quote_expr!(&*cx, ($d1, $d2)))
        .collect();
    let disps = cx.expr_vec(sp, disps);

    let entries = state
        .map
        .iter()
        .map(|&idx| {
            let &Entry {
                ref key, ref value, ..
            } = &entries[idx];
            quote_expr!(&*cx, ($key, $value))
        })
        .collect();
    let entries = cx.expr_vec(sp, entries);

    let key = state.key;
    MacEager::expr(quote_expr!(cx, ::phf::Map {
        key: $key,
        disps: &$disps,
        entries: &$entries,
    }))
}

pub fn create_set(
    cx: &mut ExtCtxt,
    sp: Span,
    entries: Vec<Entry>,
    state: HashState,
) -> Box<MacResult + 'static> {
    let map = create_map(cx, sp, entries, state).make_expr().unwrap();
    MacEager::expr(quote_expr!(cx, ::phf::Set { map: $map }))
}
