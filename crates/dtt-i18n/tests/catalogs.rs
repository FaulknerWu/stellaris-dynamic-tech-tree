use dtt_i18n::{AppLocale, Domain, Translator};

#[test]
fn locale_negotiation_matches_shared_cases() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../negotiation-cases.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let result = AppLocale::negotiate(
            case["candidates"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap()),
        );
        assert_eq!(result.tag(), case["expected"].as_str().unwrap(), "{case}");
    }
}

#[test]
fn concurrent_task_local_contexts_do_not_mix_languages_or_emit_game_bidi() {
    let tasks: Vec<_> = [AppLocale::En, AppLocale::ZhHans]
        .into_iter()
        .map(|locale| {
            std::thread::spawn(move || {
                let t = Translator::new(locale, Domain::Game).unwrap();
                for count in [0, 1, 2, 1_000_000] {
                    let message = t.tree_omitted("根 \"$tech_id$\"", count, 128).unwrap();
                    assert!(!message.contains(['\u{2068}', '\u{2069}']));
                    assert!(message.contains("$tech_id$"));
                }
                t.game_title().unwrap()
            })
        })
        .collect();
    let results: Vec<_> = tasks.into_iter().map(|task| task.join().unwrap()).collect();
    assert_eq!(results, ["Technology Tree", "科技树"]);
}
