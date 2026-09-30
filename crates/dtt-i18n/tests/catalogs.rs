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
    let tasks: Vec<_> = [
        AppLocale::En,
        AppLocale::ZhHans,
        AppLocale::Ja,
        AppLocale::Ru,
    ]
    .into_iter()
    .map(|locale| {
        std::thread::spawn(move || {
            let t = Translator::new(locale, Domain::Game).unwrap();
            for count in [0, 1, 2, 5, 11, 21, 22, 25, 1_000_000] {
                let message = t.tree_omitted("根 \"$tech_id$\"", count, 128).unwrap();
                assert!(!message.contains(['\u{2068}', '\u{2069}']));
                assert!(message.contains("$tech_id$"));
            }
            t.game_title().unwrap()
        })
    })
    .collect();
    let results: Vec<_> = tasks.into_iter().map(|task| task.join().unwrap()).collect();
    assert_eq!(
        results,
        [
            "Technology Tree",
            "科技树",
            "技術ツリー",
            "Дерево технологий"
        ]
    );
}

#[test]
fn registered_locales_round_trip_and_have_all_embedded_domains() {
    let registry: serde_json::Value = serde_json::from_str(dtt_i18n::REGISTRY).unwrap();
    for tag in registry.as_object().unwrap().keys() {
        let locale: AppLocale = serde_json::from_value(tag.clone().into()).unwrap();
        assert_eq!(locale.tag(), tag);
        assert_eq!(serde_json::to_value(locale).unwrap(), *tag);
        assert_eq!(AppLocale::matching(tag), Some(locale));
        for domain in [
            Domain::Game,
            Domain::Report,
            Domain::Cli,
            Domain::Errors,
            Domain::Diagnostics,
        ] {
            Translator::new(locale, domain).unwrap();
        }
    }
}

#[test]
fn russian_tree_omission_uses_russian_plural_rules() {
    let t = Translator::new(AppLocale::Ru, Domain::Game).unwrap();
    for (count, expected) in [
        (0, "0 непосредственно зависимых технологий"),
        (1, "1 непосредственно зависимая технология"),
        (2, "2 непосредственно зависимые технологии"),
        (5, "5 непосредственно зависимых технологий"),
        (11, "11 непосредственно зависимых технологий"),
        (21, "21 непосредственно зависимая технология"),
        (22, "22 непосредственно зависимые технологии"),
        (25, "25 непосредственно зависимых технологий"),
    ] {
        let message = t.tree_omitted("$tech_id$", count, 128).unwrap();
        assert!(message.contains(expected), "{message}");
    }
}
