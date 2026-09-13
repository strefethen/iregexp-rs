use iregexp::{Error, IRegexp, MatchMode};

fn compile(pattern: &str) -> Result<IRegexp, Error> {
    IRegexp::compile(pattern, MatchMode::Full)
}

#[test]
fn input_byte_boundaries_include_multibyte_text() {
    // A repeated class member keeps matcher size small while exercising the
    // entire parser input budget, independently of the matcher size budget.
    let at_limit = format!("[{}]", "a".repeat(65_534));
    assert_eq!(at_limit.len(), 65_536);
    assert!(compile(&at_limit).unwrap().is_match("a"));
    assert!(matches!(
        compile(&(at_limit + "a")),
        Err(Error::ResourceLimit { .. })
    ));
    let unicode_at_limit = format!("[{}]", "é".repeat(32_767));
    assert_eq!(unicode_at_limit.len(), 65_536);
    assert!(compile(&unicode_at_limit).unwrap().is_match("é"));
    assert!(matches!(
        compile(&(unicode_at_limit + "é")),
        Err(Error::ResourceLimit { .. })
    ));
}

#[test]
fn repetition_and_matcher_size_limits_are_resources() {
    assert!(compile("a{10000}").unwrap().is_match(&"a".repeat(10_000)));
    for pattern in [
        "a{10001}",
        "a{99999999999999999999999999999999999999}",
        r"\p{L}{10000}",
        "(a{10000}){10000}",
    ] {
        assert!(
            matches!(compile(pattern), Err(Error::ResourceLimit { .. })),
            "{pattern}"
        );
    }
    let padded = format!("a{{{}1}}", "0".repeat(1_000));
    assert!(compile(&padded).unwrap().is_match("a"));
}

#[test]
fn nesting_subprocess() {
    // Stack overflows abort, so isolate the probe and assert its exit status.
    // The child retains the test harness's default thread stack size.
    if std::env::var_os("IREGEXP_NESTING_CHILD").is_some() {
        for depth in [1, 32, 64, 128] {
            let pattern = format!("{}a{}", "(".repeat(depth), ")".repeat(depth));
            assert!(compile(&pattern).unwrap().is_match("a"));
        }
        for depth in [129, 1_000, 100_000] {
            let pattern = format!("{}a{}", "(".repeat(depth), ")".repeat(depth));
            assert!(matches!(
                compile(&pattern),
                Err(Error::ResourceLimit { .. })
            ));
        }
        return;
    }
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "nesting_subprocess", "--nocapture"])
        .env("IREGEXP_NESTING_CHILD", "1")
        .status()
        .unwrap();
    assert!(status.success(), "nesting child failed: {status}");
}

#[test]
fn errors_do_not_poison_reuse_or_other_threads() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<IRegexp>();
    let compiled = std::sync::Arc::new(compile("a|ab").unwrap());
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let compiled = compiled.clone();
            std::thread::spawn(move || {
                for _ in 0..10 {
                    assert!(matches!(compile("["), Err(Error::Syntax { .. })));
                    assert!(matches!(compile("[z-a]"), Err(Error::Semantic { .. })));
                    assert!(matches!(
                        compile("a{10001}"),
                        Err(Error::ResourceLimit { .. })
                    ));
                    assert!(compiled.is_match("ab"));
                    assert!(compile("a").unwrap().is_match("a"));
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
}

#[test]
fn externally_limited_parser_is_not_invalid_syntax() {
    // Pest exposes a process-global setting to all its consumers. Exercise
    // interference in a fresh child so other parallel tests remain isolated.
    if std::env::var_os("IREGEXP_PEST_LIMIT_CHILD").is_some() {
        pest::set_call_limit(std::num::NonZeroUsize::new(1));
        assert!(matches!(compile("a"), Err(Error::Backend { .. })));
        pest::set_call_limit(None);
        assert!(compile("a").unwrap().is_match("a"));
        return;
    }
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "externally_limited_parser_is_not_invalid_syntax",
            "--nocapture",
        ])
        .env("IREGEXP_PEST_LIMIT_CHILD", "1")
        .status()
        .unwrap();
    assert!(status.success(), "Pest limit child failed: {status}");
}
