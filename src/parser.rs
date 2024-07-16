use std::io::{prelude::*, BufReader, Read};

use super::environment::{Environment, InfoAnnotation, NameItem};

struct Parser {
    env: Environment,
}

type IResult<T> = Result<T, &'static str>;
type Index = usize;

fn parse_info_annotation(s: &str) -> IResult<InfoAnnotation> {
    match s {
        "#BD" => Ok(InfoAnnotation::Paren),
        "#BI" => Ok(InfoAnnotation::Curly),
        "#BS" => Ok(InfoAnnotation::DoubleCurly),
        "#BC" => Ok(InfoAnnotation::Square),
        _ => Err("Expecting info tag"),
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

fn check_eol(s: &str) -> IResult<()> {
    match next(s) {
        Some(_) => Err("Expecting EOL"),
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
        println!("Name: {}", self.env.name_to_string(idx));
    }

    fn parse_ni(&mut self, idx: Index, s: &str) -> IResult<()> {
        let (p, rest) = next_idx(s).ok_or("Expecting index")?;
        let (i, rest) = next_idx(rest).ok_or("Expecting integer")?;
        check_eol(rest)?;
        self.env.add_name(idx, NameItem::Int(i), p);
        self.post_add_name(idx);
        Ok(())
    }

    fn parse_ns(&mut self, idx: Index, s: &str) -> IResult<()> {
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

    fn post_add_univ(&self, idx: Index) {
        println!("Univ: {}", self.env.univ_to_string(idx));
    }

    fn parse_us(&mut self, idx: Index, s: &str) -> IResult<()> {
        let (u, rest) = next_idx(s).ok_or("Expecting index")?;
        check_eol(rest)?;
        self.env.add_univ_succ(idx, u);
        self.post_add_univ(idx);
        Ok(())
    }

    fn parse_um(&mut self, idx: Index, s: &str) -> IResult<()> {
        let (u1, rest) = next_idx(s).ok_or("Expecting index")?;
        let (u2, rest) = next_idx(rest).ok_or("Expecting index")?;
        check_eol(rest)?;
        self.env.add_univ_max(idx, u1, u2);
        self.post_add_univ(idx);
        Ok(())
    }

    fn parse_uim(&mut self, idx: Index, s: &str) -> IResult<()> {
        let (u1, rest) = next_idx(s).ok_or("Expecting index")?;
        let (u2, rest) = next_idx(rest).ok_or("Expecting index")?;
        check_eol(rest)?;
        self.env.add_univ_imax(idx, u1, u2);
        self.post_add_univ(idx);
        Ok(())
    }

    fn parse_up(&mut self, idx: Index, s: &str) -> IResult<()> {
        let (n, rest) = next_idx(s).ok_or("Expecting index")?;
        check_eol(rest)?;
        self.env.add_univ_param(idx, n);
        self.post_add_univ(idx);
        Ok(())
    }

    fn post_add_expr(&self, idx: Index) {
        println!("Expr: {}", self.env.expr_to_string(idx));
    }

    fn parse_es(&mut self, idx: Index, s: &str) -> IResult<()> {
        let (u, rest) = next_idx(s).ok_or("Expecting index")?;
        check_eol(rest)?;
        self.env.add_expr_sort(idx, u);
        self.post_add_expr(idx);
        Ok(())
    }

    fn parse_ev(&mut self, idx: Index, s: &str) -> IResult<()> {
        let (i, rest) = next_idx(s).ok_or("Expecting integer")?;
        check_eol(rest)?;
        self.env.add_expr_bound_var(idx, i);
        self.post_add_expr(idx);
        Ok(())
    }

    // <eidx'> #EP <info> <nidx> <eidx_1> <eidx_2>
    fn parse_ep(&mut self, idx: Index, s: &str) -> IResult<()> {
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
    fn parse_el(&mut self, idx: Index, s: &str) -> IResult<()> {
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

    fn parse_index_command(&mut self, idx: Index, s: &str) -> IResult<()> {
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
            _ => return Err("Unsupported index command"),
        }?;
        Ok(())
    }

    fn parse_command(&mut self, _s: &str) -> IResult<()> {
        Err("Invalid command")
    }

    fn parse_line(&mut self, line: &str) -> IResult<()> {
        let (first, rest) = next(line).ok_or("Expecting index or command")?;
        match first.parse::<usize>() {
            Ok(idx) => self.parse_index_command(idx, rest),
            Err(_) => self.parse_command(rest),
        }
    }

    fn get_environment(self) -> Environment {
        self.env
    }
}

pub fn parse_lines<R: Read>(file: R) -> Result<Environment, ()> {
    let reader = BufReader::new(file);

    let mut parser = Parser::new();
    let mut line_no = 1;

    for line in reader.lines() {
        let line = line.unwrap();

        if let Err(e) = parser.parse_line(&line) {
            println!("{} in line {}: {}", e, line_no, line);
            return Err(());
        }

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
