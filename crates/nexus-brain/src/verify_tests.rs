#[cfg(test)]
mod verify_tests {
    use crate::verify::CodeVerifier;

    #[test]
    fn test_verify_good_code() {
        let verifier = CodeVerifier::new();
        let code = r#"
fn process(input: Option<String>) -> Result<String, Error> {
    match input {
        Some(s) if !s.is_empty() => Ok(s),
        Some(_) => Err(Error::Empty),
        None => Err(Error::Missing),
    }
}
"#;
        let result = verifier.verify(code, "Rust project with error handling");
        assert!(result.passed || result.score > 0.5);
    }

    #[test]
    fn test_verify_bad_code() {
        let verifier = CodeVerifier::new();
        let code = r#"
fn process() {
    let x = something.unwrap();
    // TODO: fix this
}
"#;
        let result = verifier.verify(code, "");
        assert!(!result.passed);
    }

    #[test]
    fn test_syntax_check() {
        let verifier = CodeVerifier::new();
        let good = "fn test() {}";
        let bad = "fn test() {";

        let r1 = verifier.verify(good, "");
        let r2 = verifier.verify(bad, "");

        let s1 = r1.checks.iter().find(|c| c.name == "Syntax Structure").unwrap();
        let s2 = r2.checks.iter().find(|c| c.name == "Syntax Structure").unwrap();

        assert!(s1.passed);
        assert!(!s2.passed);
    }

    #[test]
    fn test_error_handling_check() {
        let verifier = CodeVerifier::new();

        // Moderate complexity (20+ lines, has fn) triggers error handling check
        let with_unwrap = r#"
fn process(input: Option<String>) -> String {
    let x = input.unwrap();
    let y = x.to_uppercase();
    let z = y.trim().to_string();
    let a = z.len();
    let b = a + 1;
    let c = b * 2;
    let d = c / 3;
    let e = d - 4;
    let f = e + 5;
    let g = f * 6;
    let h = g + 7;
    let i = h - 8;
    let j = i + 9;
    let k = j * 10;
    let l = k + 11;
    let m = l - 12;
    let n = m + 13;
    let o = n * 14;
    o
}
"#;
        let with_match = r#"
fn process(input: Option<String>) -> String {
    match input {
        Some(s) => {
            let y = s.to_uppercase();
            let z = y.trim().to_string();
            let a = z.len();
            let b = a + 1;
            let c = b * 2;
            let d = c / 3;
            let e = d - 4;
            let f = e + 5;
            let g = f * 6;
            let h = g + 7;
            let i = h - 8;
            let j = i + 9;
            let k = j * 10;
            let l = k + 11;
            let m = l - 12;
            let n = m + 13;
            let o = n * 14;
            o
        }
        None => String::new(),
    }
}
"#;

        let r1 = verifier.verify(with_unwrap, "");
        let r2 = verifier.verify(with_match, "");

        let e1 = r1.checks.iter().find(|c| c.name == "Error Handling");
        let e2 = r2.checks.iter().find(|c| c.name == "Error Handling");

        assert!(e1.is_some(), "with_unwrap should have Error Handling check, checks: {:?}", r1.checks.iter().map(|c| &c.name).collect::<Vec<_>>());
        assert!(e2.is_some(), "with_match should have Error Handling check, checks: {:?}", r2.checks.iter().map(|c| &c.name).collect::<Vec<_>>());
        assert!(!e1.unwrap().passed, "unwrap should fail: {}", e1.unwrap().details);
        assert!(e2.unwrap().passed, "match should pass: {}", e2.unwrap().details);
    }

    #[test]
    fn test_minimalism_check() {
        let verifier = CodeVerifier::new();

        let short = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
        // 200+ lines to trigger line penalty
        let lines: String = (0..210).map(|i| format!("    let x{i} = {i};\n")).collect();
        let long = format!("fn big() {{\n{lines}}}");

        let r1 = verifier.verify(short, "");
        let r2 = verifier.verify(&long, "");

        let m1 = r1.checks.iter().find(|c| c.name == "Minimalism");
        let m2 = r2.checks.iter().find(|c| c.name == "Minimalism");

        assert!(m1.is_some());
        assert!(m2.is_some());
        assert!(m1.unwrap().passed, "short should pass minimalism");
        assert!(!m2.unwrap().passed, "long should fail minimalism: {}", m2.unwrap().details);
    }
}
