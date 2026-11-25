//! Label markup conversion. Typst is canonical; LaTeX and drawio HTML are converted at the edges.

pub fn typst_to_latex(s: &str) -> String {
    if s.trim().is_empty() {
        return String::new();
    }
    tylax::typst_to_latex(s).trim().to_string()
}

pub fn latex_to_typst(s: &str) -> String {
    if s.trim().is_empty() {
        return String::new();
    }
    if !s.contains('\\') && !s.contains('$') {
        return s.to_string();
    }
    let typ = tylax::latex_document_to_typst(s)
        .lines()
        .filter(|l| !l.starts_with("#set ") && !l.starts_with("#import "))
        .collect::<Vec<_>>()
        .join("\n");
    simplify_attachments(typ.trim())
}

/// `x^(2)` -> `x^2`: tylax always parenthesizes sub- and superscripts.
fn simplify_attachments(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find(['^', '_']) {
        out.push_str(&rest[..=i]);
        rest = &rest[i + 1..];
        if let Some(inner) = rest.strip_prefix('(')
            && let Some(end) = inner.find(')')
            && end > 0
            && inner[..end].chars().all(|c| c.is_alphanumeric())
            && (inner[..end].chars().all(|c| c.is_ascii_digit()) || inner[..end].chars().count() == 1)
        {
            out.push_str(&inner[..end]);
            rest = &inner[end + 1..];
        }
    }
    out.push_str(rest);
    out
}

/// drawio labels are HTML with MathJax math (`\(..\)` inline, `$$..$$` display).
pub fn drawio_to_typst(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut tag = String::new();
    for c in s.chars() {
        match c {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                let t = tag.trim_start_matches('/').to_lowercase();
                if t.starts_with("br") || (t.starts_with("div") && !out.is_empty()) {
                    out.push('\n');
                }
            }
            _ if in_tag => tag.push(c),
            _ => out.push(c),
        }
    }
    decode_entities(&out)
        .replace("$$", "$")
        .replace("\\(", "$")
        .replace("\\)", "$")
        .lines()
        .map(latex_to_typst)
        .collect::<Vec<_>>()
        .join(" \\ ")
}

pub fn typst_to_drawio(s: &str) -> String {
    let latex = typst_to_latex(s);
    let mut out = String::new();
    let mut open = false;
    for c in latex.chars() {
        match c {
            '$' => {
                out.push_str(if open { "\\)" } else { "\\(" });
                open = !open;
            }
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out.replace("\\\\", "<br>")
}

fn decode_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn math_roundtrip() {
        assert_eq!(typst_to_latex("$a/b$"), r"$\frac{a}{b}$");
        assert_eq!(latex_to_typst(r"$\frac{a}{b}$"), "$a/b$");
        assert_eq!(latex_to_typst("plain"), "plain");
        assert_eq!(latex_to_typst(r"$x^2 + a_{ij} + b^{10}$"), "$x^2 + a_(i j) + b^10$");
    }

    #[test]
    fn drawio_labels() {
        assert_eq!(typst_to_drawio("x $alpha$"), r"x \(\alpha\)");
        assert_eq!(drawio_to_typst(r"x \(\alpha\)"), "x $alpha$");
        assert_eq!(drawio_to_typst("a<br>b"), "a \\ b");
    }
}
