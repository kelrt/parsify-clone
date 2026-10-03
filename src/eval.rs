//! Tiny expression evaluator: no dependencies, easy to extend.
//!
//! Grammar (lowest to highest precedence):
//!   expr   = term (("+" | "-") term)*
//!   term   = unary (("*" | "/" | "%") unary)*
//!   unary  = "-" unary | power
//!   power  = atom ("^" unary)?          (right-associative)
//!   atom   = number | ident | ident "(" args ")" | "(" expr ")"

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Ident(String),
    Op(char), // + - * / % ^
    LParen,
    RParen,
    Comma,
    Eq,
}

/// Variables live here. `ans` always holds the last successful result.
pub struct Env {
    pub vars: HashMap<String, f64>,
}

impl Env {
    pub fn new() -> Self {
        let mut vars = HashMap::new();
        vars.insert("pi".into(), std::f64::consts::PI);
        vars.insert("e".into(), std::f64::consts::E);
        Env { vars }
    }
}

fn tokenize(src: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    while i < chars.len() {
        let c = chars[i];
        match c {
            c if c.is_whitespace() => i += 1,
            '0'..='9' | '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '_') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().filter(|c| **c != '_').collect();
                let n = text.parse::<f64>().map_err(|_| format!("bad number '{text}'"))?;
                out.push(Tok::Num(n));
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                out.push(Tok::Ident(chars[start..i].iter().collect()));
            }
            '+' | '-' | '*' | '/' | '%' | '^' => { out.push(Tok::Op(c)); i += 1; }
            '(' => { out.push(Tok::LParen); i += 1; }
            ')' => { out.push(Tok::RParen); i += 1; }
            ',' => { out.push(Tok::Comma); i += 1; }
            '=' => { out.push(Tok::Eq); i += 1; }
            _ => return Err(format!("unexpected '{c}'")),
        }
    }
    Ok(out)
}

struct Parser<'a> {
    toks: &'a [Tok],
    pos: usize,
    env: &'a Env,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Tok> { self.toks.get(self.pos) }
    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn expr(&mut self) -> Result<f64, String> {
        let mut v = self.term()?;
        while let Some(Tok::Op(c @ ('+' | '-'))) = self.peek().cloned() {
            self.pos += 1;
            let r = self.term()?;
            v = if c == '+' { v + r } else { v - r };
        }
        Ok(v)
    }

    fn term(&mut self) -> Result<f64, String> {
        let mut v = self.unary()?;
        while let Some(Tok::Op(c @ ('*' | '/' | '%'))) = self.peek().cloned() {
            self.pos += 1;
            let r = self.unary()?;
            v = match c {
                '*' => v * r,
                '/' => {
                    if r == 0.0 { return Err("divide by zero".into()); }
                    v / r
                }
                _ => v % r,
            };
        }
        Ok(v)
    }

    fn unary(&mut self) -> Result<f64, String> {
        if let Some(Tok::Op('-')) = self.peek() {
            self.pos += 1;
            return Ok(-self.unary()?);
        }
        self.power()
    }

    fn power(&mut self) -> Result<f64, String> {
        let base = self.atom()?;
        if let Some(Tok::Op('^')) = self.peek() {
            self.pos += 1;
            let exp = self.unary()?;
            return Ok(base.powf(exp));
        }
        Ok(base)
    }

    fn atom(&mut self) -> Result<f64, String> {
        match self.next() {
            Some(Tok::Num(n)) => Ok(n),
            Some(Tok::LParen) => {
                let v = self.expr()?;
                match self.next() {
                    Some(Tok::RParen) => Ok(v),
                    _ => Err("missing ')'".into()),
                }
            }
            Some(Tok::Ident(name)) => {
                if let Some(Tok::LParen) = self.peek() {
                    self.pos += 1;
                    let mut args = Vec::new();
                    if let Some(Tok::RParen) = self.peek() {
                        self.pos += 1;
                    } else {
                        loop {
                            args.push(self.expr()?);
                            match self.next() {
                                Some(Tok::Comma) => continue,
                                Some(Tok::RParen) => break,
                                _ => return Err("missing ')'".into()),
                            }
                        }
                    }
                    call(&name, &args)
                } else {
                    self.env.vars.get(&name).copied().ok_or(format!("unknown '{name}'"))
                }
            }
            Some(t) => Err(format!("unexpected {t:?}")),
            None => Err("unexpected end".into()),
        }
    }
}

/// Add new built-in functions here.
fn call(name: &str, a: &[f64]) -> Result<f64, String> {
    let need = |n: usize| -> Result<(), String> {
        if a.len() == n { Ok(()) } else { Err(format!("{name}() takes {n} argument(s)")) }
    };
    match name {
        "sqrt" => { need(1)?; Ok(a[0].sqrt()) }
        "abs" => { need(1)?; Ok(a[0].abs()) }
        "sin" => { need(1)?; Ok(a[0].sin()) }
        "cos" => { need(1)?; Ok(a[0].cos()) }
        "tan" => { need(1)?; Ok(a[0].tan()) }
        "ln" => { need(1)?; Ok(a[0].ln()) }
        "log" => { need(1)?; Ok(a[0].log10()) }
        "round" => { need(1)?; Ok(a[0].round()) }
        "floor" => { need(1)?; Ok(a[0].floor()) }
        "ceil" => { need(1)?; Ok(a[0].ceil()) }
        "min" => a.iter().copied().reduce(f64::min).ok_or("min() needs arguments".into()),
        "max" => a.iter().copied().reduce(f64::max).ok_or("max() needs arguments".into()),
        "sum" => Ok(a.iter().sum()),
        _ => Err(format!("unknown function '{name}'")),
    }
}

/// Evaluate one line. Returns None for blank/comment-only lines.
/// Supports `name = expression` assignment and `#` comments.
pub fn eval_line(line: &str, env: &mut Env) -> Option<Result<f64, String>> {
    let code = line.split('#').next().unwrap_or("").trim();
    if code.is_empty() {
        return None;
    }
    let toks = match tokenize(code) {
        Ok(t) => t,
        Err(e) => return Some(Err(e)),
    };

    // Assignment: IDENT "=" ...
    let (target, body) = match (toks.get(0), toks.get(1)) {
        (Some(Tok::Ident(n)), Some(Tok::Eq)) => (Some(n.clone()), &toks[2..]),
        _ => (None, &toks[..]),
    };

    let mut p = Parser { toks: body, pos: 0, env };
    let result = p.expr().and_then(|v| {
        if p.pos < body.len() { Err("unexpected trailing input".into()) } else { Ok(v) }
    });

    if let Ok(v) = result {
        env.vars.insert("ans".into(), v);
        if let Some(name) = target {
            env.vars.insert(name, v);
        }
    }
    Some(result)
}

/// Format a result: up to 10 decimals, trailing zeros trimmed.
pub fn format_num(v: f64) -> String {
    if v.is_nan() { return "NaN".into(); }
    if v.is_infinite() { return if v > 0.0 { "∞".into() } else { "-∞".into() }; }
    if v != 0.0 && (v.abs() >= 1e15 || v.abs() < 1e-9) {
        return format!("{v:e}");
    }
    let s = format!("{v:.10}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Vec<String> {
        let mut env = Env::new();
        src.lines()
            .filter_map(|l| eval_line(l, &mut env))
            .map(|r| r.map(format_num).unwrap_or_else(|e| format!("err: {e}")))
            .collect()
    }

    #[test]
    fn basics() {
        assert_eq!(run("1 + 2 * 3"), ["7"]);
        assert_eq!(run("2 ^ 3 ^ 2"), ["512"]);
        assert_eq!(run("-2 ^ 2"), ["-4"]);
        assert_eq!(run("(1 + 2) * 3 # comment"), ["9"]);
        assert_eq!(run("1_000 * 2"), ["2000"]);
    }

    #[test]
    fn variables_and_ans() {
        assert_eq!(run("x = 5\ny = x * 2\nans + 1"), ["5", "10", "11"]);
        assert_eq!(run("max(1, 7, 3) + sqrt(16)"), ["11"]);
    }

    #[test]
    fn errors() {
        assert!(run("1 / 0")[0].starts_with("err"));
        assert!(run("foo + 1")[0].starts_with("err"));
    }
}
