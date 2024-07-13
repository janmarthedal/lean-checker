use core::fmt;
use std::collections::HashMap;

type NameIdx = usize;
type UnivIdx = usize;
type ExprIdx = usize;

/*
 * <nidx'> #NS <nidx> <string>
 * <nidx'> #NI <nidx> <integer>
 */

pub enum NameItem {
    Str(String),
    Int(usize),
}

impl fmt::Display for NameItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NameItem::Str(s) => s.fmt(f),
            NameItem::Int(i) => i.fmt(f),
        }
    }
}

struct Name {
    item: NameItem,
    parent: NameIdx, // Special value 0 for item with no parent
}

/*
 * <uidx'> #US  <uidx>
 * <uidx'> #UM  <uidx_1> <uidx_2>
 * <uidx'> #UIM <uidx_1> <uidx_2>
 * <uidx'> #UP  <nidx>
 */

enum Univ {
    Zero,
    Succ(UnivIdx),
    Max(UnivIdx, UnivIdx),
    IMax(UnivIdx, UnivIdx),
    Param(NameIdx),
}

/*
 * <eidx'> #EV <integer>
 * <eidx'> #ES <uidx>
 * <eidx'> #EC <nidx> <uidx>*
 * <eidx'> #EA <eidx_1> <eidx_2>
 * <eidx'> #EL <info> <nidx> <eidx_1> <eidx_2>
 * <eidx'> #EP <info> <nidx> <eidx_1> <eidx_2>
 * ---
 * <eidx'> #EJ <nidx> <integer> <eidx>
 * <eidx'> #ELN <integer>
 * <eidx'> #ELS <hex>*      // String as UTF8 bytes in hex
 */

enum InfoAnnotation {
    Paren,       // #BD
    Curly,       // #BI
    DoubleCurly, // #BS
    Square,      // #BC
}

enum Expr {
    BoundVar(usize),
    Sort(UnivIdx),
    Constant(NameIdx, Vec<UnivIdx>),
    FunAppl(ExprIdx, ExprIdx),
    Lambda(InfoAnnotation, NameIdx, ExprIdx, ExprIdx),
    Pi(InfoAnnotation, NameIdx, ExprIdx, ExprIdx),
}

pub struct Environment {
    names: HashMap<NameIdx, Name>,
    univs: HashMap<UnivIdx, Univ>,
    exprs: HashMap<ExprIdx, Expr>,
}

impl Environment {
    pub fn new() -> Self {
        let mut univs = HashMap::new();
        univs.insert(0, Univ::Zero);
        Self {
            names: HashMap::new(),
            univs,
            exprs: HashMap::new(),
        }
    }

    pub fn add_name(&mut self, idx: NameIdx, item: NameItem, parent: NameIdx) {
        let name = Name { item, parent };
        self.names.insert(idx, name);
    }

    pub fn name_to_string(&self, name_idx: NameIdx) -> String {
        let mut items: Vec<String> = Vec::new();
        let mut idx = name_idx;
        while idx != 0 {
            let item = self.names.get(&idx).expect("Name idx not found");
            items.push(item.item.to_string());
            idx = item.parent;
        }
        items.reverse();
        items.join(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        let mut env = Environment::new();
        /*
         * 1 #NS 0 foo
         * 2 #NS 1 bla
         * 3 #NI 2 1
         * 4 #NS 3 boo
         */
        env.add_name(1, NameItem::Str("foo".to_string()), 0);
        env.add_name(2, NameItem::Str("bla".to_string()), 1);
        env.add_name(3, NameItem::Int(1), 2);
        env.add_name(4, NameItem::Str("boo".to_string()), 3);
        assert_eq!(env.name_to_string(1), "foo");
        assert_eq!(env.name_to_string(2), "foo.bla");
        assert_eq!(env.name_to_string(3), "foo.bla.1");
        assert_eq!(env.name_to_string(4), "foo.bla.1.boo");
    }
}
