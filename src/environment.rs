use core::fmt;
use std::collections::HashMap;

type NameIdx = usize;
type UnivIdx = usize;
type ExprIdx = usize;

/*
 * <nidx'> #NS <nidx> <string>
 * <nidx'> #NI <nidx> <integer>
 */

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

    fn has_name(&self, idx: NameIdx) {
        assert!(self.names.contains_key(&idx));
    }

    fn has_univ(&self, idx: UnivIdx) {
        assert!(self.univs.contains_key(&idx));
    }

    pub fn add_name(&mut self, idx: NameIdx, item: NameItem, parent: NameIdx) {
        assert!(!self.names.contains_key(&idx));
        if parent != 0 {
            self.has_name(parent);
        }
        self.names.insert(idx, Name { item, parent });
    }

    pub fn name_to_string(&self, name_idx: NameIdx) -> String {
        let mut items: Vec<String> = Vec::new();
        let mut idx = name_idx;
        while idx != 0 {
            let item = self.names.get(&idx).expect("Name not found");
            items.push(item.item.to_string());
            idx = item.parent;
        }
        items.reverse();
        items.join(".")
    }

    pub fn add_univ_succ(&mut self, uidxp: UnivIdx, uidx: UnivIdx) {
        assert!(!self.univs.contains_key(&uidxp));
        self.has_univ(uidx);
        self.univs.insert(uidxp, Univ::Succ(uidx));
    }

    pub fn add_univ_max(&mut self, uidxp: UnivIdx, uidx1: UnivIdx, uidx2: UnivIdx) {
        assert!(!self.univs.contains_key(&uidxp));
        self.has_univ(uidx1);
        self.has_univ(uidx2);
        self.univs.insert(uidxp, Univ::Max(uidx1, uidx2));
    }

    pub fn add_univ_imax(&mut self, uidxp: UnivIdx, uidx1: UnivIdx, uidx2: UnivIdx) {
        assert!(!self.univs.contains_key(&uidxp));
        self.has_univ(uidx1);
        self.has_univ(uidx2);
        self.univs.insert(uidxp, Univ::IMax(uidx1, uidx2));
    }

    pub fn add_univ_param(&mut self, uidxp: UnivIdx, nidx: NameIdx) {
        assert!(!self.univs.contains_key(&uidxp));
        self.has_name(nidx);
        self.univs.insert(uidxp, Univ::Param(nidx));
    }

    pub fn univ_to_string(&self, uidx: UnivIdx) -> String {
        let univ = self.univs.get(&uidx).expect("Univ not found");
        match univ {
            Univ::Zero => "0".to_string(),
            Univ::Succ(u) => format!("(succ {})", self.univ_to_string(*u)),
            Univ::Max(u1, u2) => format!(
                "(max {} {})",
                self.univ_to_string(*u1),
                self.univ_to_string(*u2)
            ),
            Univ::IMax(u1, u2) => format!(
                "(imax {} {})",
                self.univ_to_string(*u1),
                self.univ_to_string(*u2)
            ),
            Univ::Param(n) => self.name_to_string(*n),
        }
    }

    pub fn add_expr_sort(&mut self, eidxp: ExprIdx, uidx: UnivIdx) {
        assert!(!self.exprs.contains_key(&eidxp));
        self.has_univ(uidx);
        self.exprs.insert(eidxp, Expr::Sort(uidx));
    }

    pub fn add_expr_bound_var(&mut self, eidxp: ExprIdx, i: usize) {
        assert!(!self.exprs.contains_key(&eidxp));
        self.exprs.insert(eidxp, Expr::BoundVar(i));
    }

    pub fn expr_to_string(&self, eidx: ExprIdx) -> String {
        let expr = self.exprs.get(&eidx).expect("Expr not found");
        match expr {
            Expr::Sort(u) => format!("Sort {}", self.univ_to_string(*u)),
            Expr::BoundVar(i) => format!("var[{}]", i),
            _ => todo!(),
        }
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

    #[test]
    fn universes() {
        let mut env = Environment::new();
        /*
         * 1 #NS 0 l1
         * 2 #NS 0 l2
         * 1 #US 0
         * 2 #US 1
         * 3 #UP 1
         * 4 #UP 2
         * 5 #UM 2 3
         * 6 #UIM 5 4
         */
        env.add_name(1, NameItem::Str("l1".to_string()), 0);
        env.add_name(2, NameItem::Str("l2".to_string()), 0);
        env.add_univ_succ(1, 0);
        env.add_univ_succ(2, 1);
        env.add_univ_param(3, 1);
        env.add_univ_param(4, 2);
        env.add_univ_max(5, 2, 3);
        env.add_univ_imax(6, 5, 4);
        assert_eq!(env.univ_to_string(6), "(imax (max (succ (succ 0)) l1) l2)");
    }
}
