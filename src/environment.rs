use core::fmt;
use std::collections::HashMap;

type NameIdx = usize;
type LevelIdx = usize;
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
enum Level {
    Zero,
    Succ(LevelIdx),
    Max(LevelIdx, LevelIdx),
    IMax(LevelIdx, LevelIdx),
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
 * <eidx'> #EZ <nidx> <eidx_1> <eidx_2> <eidx_3>
 */

pub enum InfoAnnotation {
    Default,         // #BD
    Implicit,        // #BI
    StrictImplicit,  // #BS
    InstImplicit,    // #BC
}

impl InfoAnnotation {
    fn to_delims(&self) -> (&'static str, &'static str) {
        match self {
            InfoAnnotation::Default => ("(", ")"),
            InfoAnnotation::Implicit => ("{", "}"),
            InfoAnnotation::StrictImplicit => ("{{", "}}"),
            InfoAnnotation::InstImplicit => ("[", "]"),
        }
    }
}

enum Expr {
    BoundVar(usize),
    Sort(LevelIdx),
    // Constant(NameIdx, Vec<UnivIdx>),
    // FunAppl(ExprIdx, ExprIdx),
    Lambda(InfoAnnotation, NameIdx, ExprIdx, ExprIdx),
    Pi(InfoAnnotation, NameIdx, ExprIdx, ExprIdx),
}

enum Const {
    // type, body, level_names
    Def(ExprIdx, ExprIdx, Vec<NameIdx>),
}

pub struct Environment {
    names: HashMap<NameIdx, Name>,
    levels: HashMap<LevelIdx, Level>,
    exprs: HashMap<ExprIdx, Expr>,
    consts: HashMap<NameIdx, Const>,
    show_var_stack: bool,
}

impl Environment {
    pub fn new() -> Self {
        let mut levels = HashMap::new();
        levels.insert(0, Level::Zero);
        Self {
            names: HashMap::new(),
            levels,
            exprs: HashMap::new(),
            consts: HashMap::new(),
            show_var_stack: false,
        }
    }

    fn has_name(&self, idx: NameIdx) {
        assert!(self.names.contains_key(&idx));
    }

    fn has_level(&self, idx: LevelIdx) {
        assert!(self.levels.contains_key(&idx));
    }

    fn has_expr(&self, idx: ExprIdx) {
        assert!(self.exprs.contains_key(&idx));
    }

    pub fn add_name(&mut self, idx: NameIdx, item: NameItem, parent: NameIdx) {
        assert!(!self.names.contains_key(&idx));
        if parent != 0 {
            self.has_name(parent);
        }
        self.names.insert(idx, Name { item, parent });
    }

    pub fn add_level_succ(&mut self, uidxp: LevelIdx, uidx: LevelIdx) {
        assert!(!self.levels.contains_key(&uidxp));
        self.has_level(uidx);
        self.levels.insert(uidxp, Level::Succ(uidx));
    }

    pub fn add_level_max(&mut self, uidxp: LevelIdx, uidx1: LevelIdx, uidx2: LevelIdx) {
        assert!(!self.levels.contains_key(&uidxp));
        self.has_level(uidx1);
        self.has_level(uidx2);
        self.levels.insert(uidxp, Level::Max(uidx1, uidx2));
    }

    pub fn add_level_imax(&mut self, uidxp: LevelIdx, uidx1: LevelIdx, uidx2: LevelIdx) {
        assert!(!self.levels.contains_key(&uidxp));
        self.has_level(uidx1);
        self.has_level(uidx2);
        self.levels.insert(uidxp, Level::IMax(uidx1, uidx2));
    }

    pub fn add_level_param(&mut self, uidxp: LevelIdx, nidx: NameIdx) {
        assert!(!self.levels.contains_key(&uidxp));
        self.has_name(nidx);
        self.levels.insert(uidxp, Level::Param(nidx));
    }

    pub fn add_expr_sort(&mut self, eidxp: ExprIdx, uidx: LevelIdx) {
        assert!(!self.exprs.contains_key(&eidxp));
        self.has_level(uidx);
        self.exprs.insert(eidxp, Expr::Sort(uidx));
    }

    pub fn add_expr_bound_var(&mut self, eidxp: ExprIdx, i: usize) {
        assert!(!self.exprs.contains_key(&eidxp));
        self.exprs.insert(eidxp, Expr::BoundVar(i));
    }

    pub fn add_expr_pi(
        &mut self,
        eidxp: ExprIdx,
        info: InfoAnnotation,
        nidx: NameIdx,
        eidx1: ExprIdx,
        eidx2: ExprIdx,
    ) {
        assert!(!self.exprs.contains_key(&eidxp));
        self.has_name(nidx);
        self.has_expr(eidx1);
        self.has_expr(eidx2);
        self.exprs.insert(eidxp, Expr::Pi(info, nidx, eidx1, eidx2));
    }

    pub fn add_expr_lambda(
        &mut self,
        eidxp: ExprIdx,
        info: InfoAnnotation,
        nidx: NameIdx,
        eidx1: ExprIdx,
        eidx2: ExprIdx,
    ) {
        assert!(!self.exprs.contains_key(&eidxp));
        self.has_name(nidx);
        self.has_expr(eidx1);
        self.has_expr(eidx2);
        self.exprs
            .insert(eidxp, Expr::Lambda(info, nidx, eidx1, eidx2));
    }

    // #DEF <nidx> <eidx_1> <edix_2> <nidx*>
    pub fn add_definition(
        &mut self,
        nidx: NameIdx,
        eidx1: ExprIdx,
        eidx2: ExprIdx,
        univ_names: Vec<NameIdx>,
    ) {
        assert!(!self.consts.contains_key(&nidx));
        self.has_name(nidx);
        self.has_expr(eidx1);
        self.has_expr(eidx2);
        univ_names.iter().for_each(|i| self.has_name(*i));
        self.consts.insert(nidx, Const::Def(eidx1, eidx2, univ_names));
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

    pub fn level_to_string(&self, uidx: LevelIdx) -> String {
        let univ = self.levels.get(&uidx).expect("Univ not found");
        match univ {
            Level::Zero => "0".to_string(),
            Level::Succ(u) => format!("(succ {})", self.level_to_string(*u)),
            Level::Max(u1, u2) => format!(
                "(max {} {})",
                self.level_to_string(*u1),
                self.level_to_string(*u2)
            ),
            Level::IMax(u1, u2) => format!(
                "(imax {} {})",
                self.level_to_string(*u1),
                self.level_to_string(*u2)
            ),
            Level::Param(n) => self.name_to_string(*n),
        }
    }

    pub fn expr_to_string(&self, eidx: ExprIdx) -> String {
        let mut var_stack: Vec<String> = vec![];
        let result = self.expr_to_string_help(eidx, &mut var_stack);
        assert!(var_stack.is_empty());
        result
    }

    fn pi_or_lambda_to_string(
        &self,
        info: &InfoAnnotation,
        nidx: NameIdx,
        eidx1: ExprIdx,
        eidx2: ExprIdx,
        var_stack: &mut Vec<String>,
    ) -> String {
        let delims = info.to_delims();
        let var_name = self.name_to_string(nidx);
        let e1 = self.expr_to_string_help(eidx1, var_stack);
        var_stack.push(var_name.clone());
        let e2 = self.expr_to_string_help(eidx2, var_stack);
        var_stack.pop();
        let mut result = format!("{}{} : {}{}, {}", delims.0, var_name, e1, delims.1, e2);
        if self.show_var_stack {
            result.push_str(&format!(" [{}]", var_stack.join(",")));
        }
        result
    }

    fn expr_to_string_help(&self, eidx: ExprIdx, var_stack: &mut Vec<String>) -> String {
        let expr = self.exprs.get(&eidx).expect("Expr not found");
        match expr {
            Expr::Sort(u) => format!("Sort {}", self.level_to_string(*u)),
            Expr::BoundVar(i) => if *i < var_stack.len() {
                var_stack[var_stack.len() - 1 - *i].clone()
            } else {
                format!("<{}>", i)
            },
            Expr::Pi(info, n, i1, i2) => self.pi_or_lambda_to_string(info, *n, *i1, *i2, var_stack),
            Expr::Lambda(info, n, i1, i2) => {
                self.pi_or_lambda_to_string(info, *n, *i1, *i2, var_stack)
            }
        }
    }

    fn def_to_string(&self, name: &String, eidx1: ExprIdx, eidx2: ExprIdx, level_name_idxs: &Vec<NameIdx>) -> String {
        let univ_names = level_name_idxs
            .iter()
            .map(|ni| self.name_to_string(*ni))
            .collect::<Vec<String>>()
            .join(",");
        let univ_names_fmt = if univ_names.is_empty() {
            "".to_string()
        } else {
            format!(".{{{}}}", univ_names)
        };
        let type_expr = self.expr_to_string(eidx1);
        let body_expr = self.expr_to_string(eidx2);
        format!(
            "definition {}{} {} := {}",
            name, univ_names_fmt, type_expr, body_expr
        )
    }

    pub fn constant_to_string(&self, nidx: NameIdx) -> String {
        let cnst = self.consts.get(&nidx).expect("Constant not found");
        let name = self.name_to_string(nidx);
        match cnst {
            Const::Def(eidx1, eidx2, level_names) => {
                self.def_to_string(&name, *eidx1, *eidx2, level_names)
            }
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
    fn levels() {
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
        env.add_level_succ(1, 0);
        env.add_level_succ(2, 1);
        env.add_level_param(3, 1);
        env.add_level_param(4, 2);
        env.add_level_max(5, 2, 3);
        env.add_level_imax(6, 5, 4);
        assert_eq!(env.level_to_string(6), "(imax (max (succ (succ 0)) l1) l2)");
    }
}
