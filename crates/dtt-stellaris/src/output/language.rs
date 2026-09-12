use super::SupportedLanguage;

pub struct LangStrings {
    pub title: &'static str,
    pub top_level: &'static str,
    pub tier_label: &'static str,
    pub requires: &'static str,
}

const ENGLISH: LangStrings = LangStrings {
    title: "Technology Tree",
    top_level: "Maximum Level Reached",
    tier_label: "Tier:",
    requires: "Requires",
};

const SIMP_CHINESE: LangStrings = LangStrings {
    title: "科技树",
    top_level: "已达到顶级",
    tier_label: "级别:",
    requires: "还需",
};

const FRENCH: LangStrings = LangStrings {
    title: "Arbre Technologique",
    top_level: "Niveau maximum atteint",
    tier_label: "Niveau:",
    requires: "Requiert",
};

const GERMAN: LangStrings = LangStrings {
    title: "Technologiebaum",
    top_level: "Maximale Stufe erreicht",
    tier_label: "Stufe:",
    requires: "Benötigt",
};

const SPANISH: LangStrings = LangStrings {
    title: "Árbol Tecnológico",
    top_level: "Nivel Máximo Alcanzado",
    tier_label: "Nivel:",
    requires: "Requiere",
};

const RUSSIAN: LangStrings = LangStrings {
    title: "Древо технологий",
    top_level: "Достигнут максимальный уровень",
    tier_label: "Уровень:",
    requires: "Требуется",
};

const KOREAN: LangStrings = LangStrings {
    title: "기술 트리",
    top_level: "최대 단계 도달",
    tier_label: "단계:",
    requires: "요구",
};

const JAPANESE: LangStrings = LangStrings {
    title: "技術ツリー",
    top_level: "最大レベルに到達",
    tier_label: "レベル:",
    requires: "必要",
};

const POLISH: LangStrings = LangStrings {
    title: "Drzewo Technologii",
    top_level: "Osiągnięto maksymalny poziom",
    tier_label: "Poziom:",
    requires: "Wymaga",
};

const BRAZ_POR: LangStrings = LangStrings {
    title: "Árvore de Tecnologia",
    top_level: "Nível Máximo Atingido",
    tier_label: "Nível:",
    requires: "Requer",
};

pub(crate) const fn strings_for(language: SupportedLanguage) -> &'static LangStrings {
    match language {
        SupportedLanguage::English => &ENGLISH,
        SupportedLanguage::SimpChinese => &SIMP_CHINESE,
        SupportedLanguage::French => &FRENCH,
        SupportedLanguage::German => &GERMAN,
        SupportedLanguage::Spanish => &SPANISH,
        SupportedLanguage::Russian => &RUSSIAN,
        SupportedLanguage::Korean => &KOREAN,
        SupportedLanguage::Japanese => &JAPANESE,
        SupportedLanguage::Polish => &POLISH,
        SupportedLanguage::BrazPor => &BRAZ_POR,
    }
}
