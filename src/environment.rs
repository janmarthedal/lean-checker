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
    Default,        // #BD
    Implicit,       // #BI
    StrictImplicit, // #BS
    InstImplicit,   // #BC
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
    Constant(NameIdx, Vec<LevelIdx>),
    FunAppl(ExprIdx, ExprIdx),
    Lambda(InfoAnnotation, NameIdx, ExprIdx, ExprIdx),
    Pi(InfoAnnotation, NameIdx, ExprIdx, ExprIdx),
}

// #AX <nidx> <eidx> <nidx*>
// #DEF <nidx> <eidx_1> <edix_2> <nidx*>
// #IND <num> <nidx> <eidx> <num_intros> <intro>* <nidx*>
enum Decl {
    // type, body, level_names
    Def(ExprIdx, ExprIdx, Vec<NameIdx>),
    // parameters, name, type, introduction rules, and universe parameters
    Ind(usize, ExprIdx, Vec<(NameIdx, ExprIdx)>, Vec<NameIdx>),
}

pub struct Environment {
    names: HashMap<NameIdx, Name>,
    levels: HashMap<LevelIdx, Level>,
    exprs: HashMap<ExprIdx, Expr>,
    decls: HashMap<NameIdx, Decl>,
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
            decls: HashMap::new(),
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

    pub fn add_expr_constant(&mut self, eidxp: ExprIdx, nidx: NameIdx, level_idxs: Vec<LevelIdx>) {
        assert!(!self.exprs.contains_key(&eidxp));
        self.has_name(nidx);
        level_idxs.iter().for_each(|li| self.has_level(*li));
        self.exprs.insert(eidxp, Expr::Constant(nidx, level_idxs));
    }

    pub fn add_expr_funappl(&mut self, eidxp: ExprIdx, eidx1: ExprIdx, eidx2: ExprIdx) {
        assert!(!self.exprs.contains_key(&eidxp));
        self.has_expr(eidx1);
        self.has_expr(eidx2);
        self.exprs.insert(eidxp, Expr::FunAppl(eidx1, eidx2));
    }

    // #DEF <nidx> <eidx_1> <edix_2> <nidx*>
    pub fn add_definition(
        &mut self,
        nidx: NameIdx,
        eidx1: ExprIdx,
        eidx2: ExprIdx,
        level_names: Vec<NameIdx>,
    ) {
        assert!(!self.decls.contains_key(&nidx));
        self.has_name(nidx);
        self.has_expr(eidx1);
        self.has_expr(eidx2);
        level_names.iter().for_each(|i| self.has_name(*i));
        self.decls
            .insert(nidx, Decl::Def(eidx1, eidx2, level_names));
    }

    // #IND <num> <nidx> <eidx> <num_intros> <intro>* <nidx*>
    pub fn add_inductive(
        &mut self,
        params: usize,
        nidx: NameIdx,
        eidx: ExprIdx,
        intros: Vec<(NameIdx, ExprIdx)>,
        level_names: Vec<NameIdx>,
    ) {
        assert!(!self.decls.contains_key(&nidx));
        self.has_expr(eidx);
        intros.iter().for_each(|(ni, ei)| {
            self.has_name(*ni);
            self.has_expr(*ei);
        });
        level_names.iter().for_each(|i| self.has_name(*i));
        self.decls
            .insert(nidx, Decl::Ind(params, eidx, intros, level_names));
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
            Expr::BoundVar(i) => {
                if *i < var_stack.len() {
                    var_stack[var_stack.len() - 1 - *i].clone()
                } else {
                    format!("<{}>", i)
                }
            }
            Expr::Pi(info, n, i1, i2) => self.pi_or_lambda_to_string(info, *n, *i1, *i2, var_stack),
            Expr::Lambda(info, n, i1, i2) => {
                self.pi_or_lambda_to_string(info, *n, *i1, *i2, var_stack)
            }
            Expr::Constant(n, lvls) => {
                let name = self.name_to_string(*n);
                if lvls.is_empty() {
                    name
                } else {
                    format!(
                        "{}.{{{}}}",
                        name,
                        lvls.iter()
                            .map(|li| self.level_to_string(*li))
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                }
            }
            Expr::FunAppl(fe, be) => {
                let fst = self.expr_to_string_help(*fe, var_stack);
                let bst = self.expr_to_string_help(*be, var_stack);
                format!("({} {})", fst, bst)
            }
        }
    }

    fn def_to_string(
        &self,
        name: &String,
        eidx1: ExprIdx,
        eidx2: ExprIdx,
        level_name_idxs: &[NameIdx],
    ) -> String {
        let level_names = level_name_idxs
            .iter()
            .map(|ni| self.name_to_string(*ni))
            .collect::<Vec<String>>()
            .join(",");
        let level_names_fmt = if level_names.is_empty() {
            "".to_string()
        } else {
            format!(".{{{}}}", level_names)
        };
        let type_expr = self.expr_to_string(eidx1);
        let body_expr = self.expr_to_string(eidx2);
        format!(
            "definition {}{} {} := {}",
            name, level_names_fmt, type_expr, body_expr
        )
    }

    fn ind_to_string(
        &self,
        name: &String,
        eidx: ExprIdx,
        intros: &[(NameIdx, ExprIdx)],
        level_name_idxs: &[LevelIdx],
    ) -> String {
        let type_expr = self.expr_to_string(eidx);
        let level_names = level_name_idxs
            .iter()
            .map(|ni| self.name_to_string(*ni))
            .collect::<Vec<String>>()
            .join(",");
        let level_names_fmt = if level_names.is_empty() {
            "".to_string()
        } else {
            format!(" {{{}}}", level_names)
        };
        let intros_fmt = intros
            .iter()
            .map(|(ni, ei)| {
                format!(
                    "\n| {} : {}",
                    self.name_to_string(*ni),
                    self.expr_to_string(*ei)
                )
            })
            .collect::<Vec<String>>()
            .join(" ");
        format!(
            "inductive {}{} {}{}",
            name, level_names_fmt, type_expr, intros_fmt
        )
    }

    pub fn decl_to_string(&self, nidx: NameIdx) -> String {
        let decl = self.decls.get(&nidx).expect("Declaration not found");
        let name = self.name_to_string(nidx);
        match decl {
            Decl::Def(eidx1, eidx2, level_names) => {
                self.def_to_string(&name, *eidx1, *eidx2, level_names)
            }
            Decl::Ind(_params, eidx, intros, level_names) => {
                self.ind_to_string(&name, *eidx, intros, level_names)
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
