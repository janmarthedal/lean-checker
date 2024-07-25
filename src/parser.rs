use std::io::{prelude::*, BufReader, Read};

use super::environment::{Environment, InfoAnnotation, NameItem};

struct LineError {
    msg: String,
}

impl From<&str> for LineError {
    fn from(value: &str) -> Self {
        Self {
            msg: String::from(value),
        }
    }
}

impl From<std::io::Error> for LineError {
    fn from(err: std::io::Error) -> Self {
        Self {
            msg: err.to_string(),
        }
    }
}

impl std::fmt::Display for LineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.msg.fmt(f)
    }
}

pub struct ParseError {
    line_error: LineError,
    line_no: usize,
}

impl ParseError {
    fn new(line_error: LineError, line_no: usize) -> Self {
        Self {
            line_error,
            line_no,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error at line {}: {}",
            self.line_no, self.line_error
        )
    }
}

struct Parser {
    env: Environment,
}

type LineResult<T> = std::result::Result<T, LineError>;
type Index = usize;

fn parse_info_annotation(s: &str) -> LineResult<InfoAnnotation> {
    match s {
        "#BD" => Ok(InfoAnnotation::Default),
        "#BI" => Ok(InfoAnnotation::Implicit),
        "#BS" => Ok(InfoAnnotation::StrictImplicit),
        "#BC" => Ok(InfoAnnotation::InstImplicit),
        _ => Err(LineError::from("Expecting info tag")),
    }
}

fn next(s: &str) -> Option<(&str, &str)> {
    s.find(|c| !char::is_whitespace(c)).map(|p| {
        let s = &s[p..];
        s.split_at(s.find(char::is_whitespace).unwrap_or(s.len()))
    })
}

fn next_idx(s: &str) -> Option<(Index, &str)> {
    match next(s) {
        Some((t, r)) => t.parse::<usize>().map(|i| (i, r)).ok(),
        None => None,
    }
}

fn check_eol(s: &str) -> LineResult<()> {
    match next(s) {
        Some(_) => Err(LineError::from("Expecting EOL")),
        None => Ok(()),
    }
}

impl Parser {
    fn new() -> Self {
        Self {
            env: Environment::new(),
        }
    }

    fn post_add_name(&self, idx: Index) {
        println!("Name {}: {}", idx, self.env.name_to_string(idx));
    }

    fn parse_ni(&mut self, idx: Index, s: &str) -> LineResult<()> {
        let (p, rest) = next_idx(s).ok_or("Expecting index")?;
        let (i, rest) = next_idx(rest).ok_or("Expecting integer")?;
        check_eol(rest)?;
        self.env.add_name(idx, NameItem::Int(i), p);
        self.post_add_name(idx);
        Ok(())
    }

    fn parse_ns(&mut self, idx: Index, s: &str) -> LineResult<()> {
        let (p, rest) = next_idx(s).ok_or("Expecting index")?;
        let (s, rest) = next(rest).ok_or("Expecting identifier")?;
        check_eol(rest)?;
        self.env.add_name(idx, NameItem::Str(s.to_string()), p);
        self.post_add_name(idx);
        Ok(())
    }

    /*
     * <uidx'> #US  <uidx>
     * <uidx'> #UM  <uidx_1> <uidx_2>
     * <uidx'> #UIM <uidx_1> <uidx_2>
     * <uidx'> #UP  <nidx>
     */

    fn post_add_level(&self, idx: Index) {
        println!("Level {}: {}", idx, self.env.level_to_string(idx));
    }

    fn parse_us(&mut self, idx: Index, s: &str) -> LineResult<()> {
        let (u, rest) = next_idx(s).ok_or("Expecting index")?;
        check_eol(rest)?;
        self.env.add_level_succ(idx, u);
        self.post_add_level(idx);
        Ok(())
    }

    fn parse_um(&mut self, idx: Index, s: &str) -> LineResult<()> {
        let (u1, rest) = next_idx(s).ok_or("Expecting index")?;
        let (u2, rest) = next_idx(rest).ok_or("Expecting index")?;
        check_eol(rest)?;
        self.env.add_level_max(idx, u1, u2);
        self.post_add_level(idx);
        Ok(())
    }

    fn parse_uim(&mut self, idx: Index, s: &str) -> LineResult<()> {
        let (u1, rest) = next_idx(s).ok_or("Expecting index")?;
        let (u2, rest) = next_idx(rest).ok_or("Expecting index")?;
        check_eol(rest)?;
        self.env.add_level_imax(idx, u1, u2);
        self.post_add_level(idx);
        Ok(())
    }

    fn parse_up(&mut self, idx: Index, s: &str) -> LineResult<()> {
        let (n, rest) = next_idx(s).ok_or("Expecting index")?;
        check_eol(rest)?;
        self.env.add_level_param(idx, n);
        self.post_add_level(idx);
        Ok(())
    }

    fn post_add_expr(&self, idx: Index) {
        println!("Expr {}: {}", idx, self.env.expr_to_string(idx));
    }

    fn parse_es(&mut self, idx: Index, s: &str) -> LineResult<()> {
        let (u, rest) = next_idx(s).ok_or("Expecting index")?;
        check_eol(rest)?;
        self.env.add_expr_sort(idx, u);
        self.post_add_expr(idx);
        Ok(())
    }

    fn parse_ev(&mut self, idx: Index, s: &str) -> LineResult<()> {
        let (i, rest) = next_idx(s).ok_or("Expecting integer")?;
        check_eol(rest)?;
        self.env.add_expr_bound_var(idx, i);
        self.post_add_expr(idx);
        Ok(())
    }

    // <eidx'> #EP <info> <nidx> <eidx_1> <eidx_2>
    fn parse_ep(&mut self, idx: Index, s: &str) -> LineResult<()> {
        let (info, rest) = next(s).ok_or("Expecting info")?;
        let info = parse_info_annotation(info)?;
        let (nidx, rest) = next_idx(rest).ok_or("Expecting index")?;
        let (eidx1, rest) = next_idx(rest).ok_or("Expecting index")?;
        let (eidx2, rest) = next_idx(rest).ok_or("Expecting index")?;
        check_eol(rest)?;
        self.env.add_expr_pi(idx, info, nidx, eidx1, eidx2);
        self.post_add_expr(idx);
        Ok(())
    }

    // <eidx'> #EL <info> <nidx> <eidx_1> <eidx_2>
    fn parse_el(&mut self, idx: Index, s: &str) -> LineResult<()> {
        let (info, rest) = next(s).ok_or("Expecting info")?;
        let info = parse_info_annotation(info)?;
        let (nidx, rest) = next_idx(rest).ok_or("Expecting index")?;
        let (eidx1, rest) = next_idx(rest).ok_or("Expecting index")?;
        let (eidx2, rest) = next_idx(rest).ok_or("Expecting index")?;
        check_eol(rest)?;
        self.env.add_expr_lambda(idx, info, nidx, eidx1, eidx2);
        self.post_add_expr(idx);
        Ok(())
    }

    // <eidx'> #EC <nidx> <uidx>*
    fn parse_ec(&mut self, idx: Index, s: &str) -> LineResult<()> {
        let (nidx, rest) = next_idx(s).ok_or("Expecting index")?;
        let mut level_idxs: Vec<Index> = vec![];
        let mut rest = rest;
        while let Some((ni, r)) = next_idx(rest) {
            level_idxs.push(ni);
            rest = r;
        }
        check_eol(rest)?;
        self.env.add_expr_constant(idx, nidx, level_idxs);
        self.post_add_expr(idx);
        Ok(())
    }

    // <eidx'> #EA <eidx_1> <eidx_2>
    fn parse_ea(&mut self, idx: Index, s: &str) -> LineResult<()> {
        let (eidx1, rest) = next_idx(s).ok_or("Expecting index")?;
        let (eidx2, rest) = next_idx(rest).ok_or("Expecting index")?;
        check_eol(rest)?;
        self.env.add_expr_funappl(idx, eidx1, eidx2);
        self.post_add_expr(idx);
        Ok(())
    }

    fn post_add_declaration(&self, idx: Index) {
        println!("Declaration {}: {}", idx, self.env.decl_to_string(idx));
    }

    // #DEF <nidx> <eidx_1> <edix_2> <nidx*>
    fn parse_def(&mut self, s: &str) -> LineResult<()> {
        let (nidx, rest) = next_idx(s).ok_or("Expecting index")?;
        let (eidx1, rest) = next_idx(rest).ok_or("Expecting index")?;
        let (eidx2, rest) = next_idx(rest).ok_or("Expecting index")?;
        let mut level_nidxs: Vec<Index> = vec![];
        let mut rest = rest;
        while let Some((ni, r)) = next_idx(rest) {
            level_nidxs.push(ni);
            rest = r;
        }
        check_eol(rest)?;
        self.env.add_definition(nidx, eidx1, eidx2, level_nidxs);
        self.post_add_declaration(nidx);
        Ok(())
    }

    // #IND <num> <nidx> <eidx> <num_intros> <intro>* <nidx*>
    fn parse_ind(&mut self, s: &str) -> LineResult<()> {
        let (num, rest) = next_idx(s).ok_or("Expecting number")?;
        let (nidx, rest) = next_idx(rest).ok_or("Expecting index")?;
        let (eidx, rest) = next_idx(rest).ok_or("Expecting index")?;
        let (num_intros, rest) = next_idx(rest).ok_or("Expecting number")?;
        let mut rest = rest;
        let mut intros: Vec<(Index, Index)> = vec![];
        for _ in 0..num_intros {
            let (ni, r) = next_idx(rest).ok_or("Expecting index")?;
            let (ei, r) = next_idx(r).ok_or("Expecting index")?;
            intros.push((ni, ei));
            rest = r;
        }
        let mut level_nidxs: Vec<Index> = vec![];
        while let Some((ni, r)) = next_idx(rest) {
            level_nidxs.push(ni);
            rest = r;
        }
        check_eol(rest)?;
        self.env.add_inductive(num, nidx, eidx, intros, level_nidxs);
        self.post_add_declaration(nidx);
        Ok(())
    }

    fn parse_index_command(&mut self, idx: Index, s: &str) -> LineResult<()> {
        let (cmd, rest) = next(s).ok_or("Expecting index command")?;
        match cmd {
            "#NI" => self.parse_ni(idx, rest),
            "#NS" => self.parse_ns(idx, rest),

            "#US" => self.parse_us(idx, rest),
            "#UM" => self.parse_um(idx, rest),
            "#UIM" => self.parse_uim(idx, rest),
            "#UP" => self.parse_up(idx, rest),

            "#ES" => self.parse_es(idx, rest),
            "#EV" => self.parse_ev(idx, rest),
            "#EP" => self.parse_ep(idx, rest),
            "#EL" => self.parse_el(idx, rest),
            "#EC" => self.parse_ec(idx, rest),
            "#EA" => self.parse_ea(idx, rest),
            "#EJ" => todo!("#EJ"),
            "#ELN" => todo!("#ELN"),
            "#ELS" => todo!("#ELS"),
            "#EZ" => todo!("#EZ"),

            _ => return Err(LineError::from("Unsupported index command")),
        }?;
        Ok(())
    }

    fn parse_command(&mut self, cmd: &str, rest: &str) -> LineResult<()> {
        match cmd {
            "#DEF" => self.parse_def(rest),
            "#AX" => todo!("#AX"),
            "#IND" => self.parse_ind(rest),
            "#QUOT" => todo!("#QUOT"),
            "#PREFIX" => todo!("#PREFIX"),
            "#POSTFIX" => todo!("#POSTFIX"),
            "#INFIX" => todo!("#INFIX"),

            _ => return Err(LineError::from("Unsupported command")),
        }?;
        Ok(())
    }

    fn parse_line(&mut self, line: &str) -> LineResult<()> {
        let (first, rest) = next(line).ok_or("Expecting index or command")?;
        match first.parse::<usize>() {
            Ok(idx) => self.parse_index_command(idx, rest),
            Err(_) => self.parse_command(first, rest),
        }
    }

    fn get_environment(self) -> Environment {
        self.env
    }
}

pub fn parse_lines<R: Read>(file: R) -> std::result::Result<Environment, ParseError> {
    let reader = BufReader::new(file);

    let mut parser = Parser::new();
    let mut line_no = 1;

    for line in reader.lines() {
        let line = line.map_err(|e| ParseError::new(LineError::from(e), line_no))?;

        parser
            .parse_line(&line)
            .map_err(|line_error| ParseError::new(line_error, line_no))?;

        line_no += 1;
    }

    Ok(parser.get_environment())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next() {
        let res = next(" a  foo ba  ");
        assert_eq!(res, Some(("a", "  foo ba  ")));
        let res = next(res.unwrap().1);
        assert_eq!(res, Some(("foo", " ba  ")));
        let res = next(res.unwrap().1);
        assert_eq!(res, Some(("ba", "  ")));
        let res = next(res.unwrap().1);
        assert!(res.is_none());
    }

    #[test]
    fn test_next_idx() {
        let res = next_idx(" 1  234 56  ");
        assert_eq!(res, Some((1, "  234 56  ")));
        let res = next_idx(res.unwrap().1);
        assert_eq!(res, Some((234, " 56  ")));
        let res = next_idx(res.unwrap().1);
        assert_eq!(res, Some((56, "  ")));
        let res = next_idx(res.unwrap().1);
        assert!(res.is_none());
    }
}
