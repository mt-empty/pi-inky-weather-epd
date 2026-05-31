use chrono::{DateTime, Datelike, Local, Weekday};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    Fr,
    De,
    Es,
    Ja,
}

impl Language {
    pub fn from_config(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "fr" => Self::Fr,
            "de" => Self::De,
            "es" => Self::Es,
            "ja" => Self::Ja,
            _ => Self::En,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationKey {
    Feels,
    Like,
    Metric,
    Now,
    Max,
    Hours24,
    Tomorrow,
}

pub fn translate(key: TranslationKey, language_code: &str) -> &'static str {
    match (Language::from_config(language_code), key) {
        (Language::En, TranslationKey::Feels) => "Feels",
        (Language::En, TranslationKey::Like) => "Like",
        (Language::En, TranslationKey::Metric) => "Metric",
        (Language::En, TranslationKey::Now) => "Now",
        (Language::En, TranslationKey::Max) => "Max",
        (Language::En, TranslationKey::Hours24) => "24h",
        (Language::En, TranslationKey::Tomorrow) => "Tomorrow",
        (Language::Fr, TranslationKey::Feels) => "Ress.",
        (Language::Fr, TranslationKey::Like) => "comme",
        (Language::Fr, TranslationKey::Metric) => "Mesure",
        (Language::Fr, TranslationKey::Now) => "Maint.",
        (Language::Fr, TranslationKey::Max) => "Max",
        (Language::Fr, TranslationKey::Hours24) => "24h",
        (Language::Fr, TranslationKey::Tomorrow) => "Demain",
        (Language::De, TranslationKey::Feels) => "Gef.",
        (Language::De, TranslationKey::Like) => "wie",
        (Language::De, TranslationKey::Metric) => "Wert",
        (Language::De, TranslationKey::Now) => "Jetzt",
        (Language::De, TranslationKey::Max) => "Max",
        (Language::De, TranslationKey::Hours24) => "24h",
        (Language::De, TranslationKey::Tomorrow) => "Morgen",
        (Language::Es, TranslationKey::Feels) => "Se",
        (Language::Es, TranslationKey::Like) => "siente",
        (Language::Es, TranslationKey::Metric) => "Medida",
        (Language::Es, TranslationKey::Now) => "Ahora",
        (Language::Es, TranslationKey::Max) => "Max",
        (Language::Es, TranslationKey::Hours24) => "24h",
        (Language::Es, TranslationKey::Tomorrow) => "Mañana",
        (Language::Ja, TranslationKey::Feels) => "Taikan",
        (Language::Ja, TranslationKey::Like) => "ondo",
        (Language::Ja, TranslationKey::Metric) => "Shihyo",
        (Language::Ja, TranslationKey::Now) => "Ima",
        (Language::Ja, TranslationKey::Max) => "Saidai",
        (Language::Ja, TranslationKey::Hours24) => "24h",
        (Language::Ja, TranslationKey::Tomorrow) => "Ashita",
    }
}

pub fn weekday_short(weekday: Weekday, language_code: &str) -> &'static str {
    match Language::from_config(language_code) {
        Language::En => match weekday {
            Weekday::Mon => "Mon",
            Weekday::Tue => "Tue",
            Weekday::Wed => "Wed",
            Weekday::Thu => "Thu",
            Weekday::Fri => "Fri",
            Weekday::Sat => "Sat",
            Weekday::Sun => "Sun",
        },
        Language::Fr => match weekday {
            Weekday::Mon => "Lun",
            Weekday::Tue => "Mar",
            Weekday::Wed => "Mer",
            Weekday::Thu => "Jeu",
            Weekday::Fri => "Ven",
            Weekday::Sat => "Sam",
            Weekday::Sun => "Dim",
        },
        Language::De => match weekday {
            Weekday::Mon => "Mo",
            Weekday::Tue => "Di",
            Weekday::Wed => "Mi",
            Weekday::Thu => "Do",
            Weekday::Fri => "Fr",
            Weekday::Sat => "Sa",
            Weekday::Sun => "So",
        },
        Language::Es => match weekday {
            Weekday::Mon => "Lun",
            Weekday::Tue => "Mar",
            Weekday::Wed => "Mie",
            Weekday::Thu => "Jue",
            Weekday::Fri => "Vie",
            Weekday::Sat => "Sab",
            Weekday::Sun => "Dom",
        },
        Language::Ja => match weekday {
            Weekday::Mon => "Getsu",
            Weekday::Tue => "Ka",
            Weekday::Wed => "Sui",
            Weekday::Thu => "Moku",
            Weekday::Fri => "Kin",
            Weekday::Sat => "Do",
            Weekday::Sun => "Nichi",
        },
    }
}

pub fn weekday_long(weekday: Weekday, language_code: &str) -> &'static str {
    match Language::from_config(language_code) {
        Language::En => match weekday {
            Weekday::Mon => "Monday",
            Weekday::Tue => "Tuesday",
            Weekday::Wed => "Wednesday",
            Weekday::Thu => "Thursday",
            Weekday::Fri => "Friday",
            Weekday::Sat => "Saturday",
            Weekday::Sun => "Sunday",
        },
        Language::Fr => match weekday {
            Weekday::Mon => "Lundi",
            Weekday::Tue => "Mardi",
            Weekday::Wed => "Mercredi",
            Weekday::Thu => "Jeudi",
            Weekday::Fri => "Vendredi",
            Weekday::Sat => "Samedi",
            Weekday::Sun => "Dimanche",
        },
        Language::De => match weekday {
            Weekday::Mon => "Montag",
            Weekday::Tue => "Dienstag",
            Weekday::Wed => "Mittwoch",
            Weekday::Thu => "Donnerstag",
            Weekday::Fri => "Freitag",
            Weekday::Sat => "Samstag",
            Weekday::Sun => "Sonntag",
        },
        Language::Es => match weekday {
            Weekday::Mon => "Lunes",
            Weekday::Tue => "Martes",
            Weekday::Wed => "Miércoles",
            Weekday::Thu => "Jueves",
            Weekday::Fri => "Viernes",
            Weekday::Sat => "Sábado",
            Weekday::Sun => "Domingo",
        },
        Language::Ja => match weekday {
            Weekday::Mon => "Getsuyobi",
            Weekday::Tue => "Kayobi",
            Weekday::Wed => "Suiyobi",
            Weekday::Thu => "Mokuyobi",
            Weekday::Fri => "Kinyobi",
            Weekday::Sat => "Doyobi",
            Weekday::Sun => "Nichiyobi",
        },
    }
}

pub fn month_short(month: u32, language_code: &str) -> &'static str {
    match Language::from_config(language_code) {
        Language::En => match month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "",
        },
        Language::Fr => match month {
            1 => "Janv",
            2 => "Févr",
            3 => "Mars",
            4 => "Avr",
            5 => "Mai",
            6 => "Juin",
            7 => "Juil",
            8 => "Août",
            9 => "Sept",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "",
        },
        Language::De => match month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "Mai",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Okt",
            11 => "Nov",
            12 => "Dez",
            _ => "",
        },
        Language::Es => match month {
            1 => "Ene",
            2 => "Feb",
            3 => "Mar",
            4 => "Abr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Ago",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dic",
            _ => "",
        },
        Language::Ja => match month {
            1 => "1gatsu",
            2 => "2gatsu",
            3 => "3gatsu",
            4 => "4gatsu",
            5 => "5gatsu",
            6 => "6gatsu",
            7 => "7gatsu",
            8 => "8gatsu",
            9 => "9gatsu",
            10 => "10gatsu",
            11 => "11gatsu",
            12 => "12gatsu",
            _ => "",
        },
    }
}

pub fn month_long(month: u32, language_code: &str) -> &'static str {
    match Language::from_config(language_code) {
        Language::En => match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "",
        },
        Language::Fr => match month {
            1 => "Janvier",
            2 => "Février",
            3 => "Mars",
            4 => "Avril",
            5 => "Mai",
            6 => "Juin",
            7 => "Juillet",
            8 => "Août",
            9 => "Septembre",
            10 => "Octobre",
            11 => "Novembre",
            12 => "Decembre",
            _ => "",
        },
        Language::De => match month {
            1 => "Januar",
            2 => "Februar",
            3 => "März",
            4 => "April",
            5 => "Mai",
            6 => "Juni",
            7 => "Juli",
            8 => "August",
            9 => "September",
            10 => "Oktober",
            11 => "November",
            12 => "Dezember",
            _ => "",
        },
        Language::Es => match month {
            1 => "Enero",
            2 => "Febrero",
            3 => "Marzo",
            4 => "Abril",
            5 => "Mayo",
            6 => "Junio",
            7 => "Julio",
            8 => "Agosto",
            9 => "Septiembre",
            10 => "Octubre",
            11 => "Noviembre",
            12 => "Diciembre",
            _ => "",
        },
        Language::Ja => match month {
            1 => "Ichigatsu",
            2 => "Nigatsu",
            3 => "Sangatsu",
            4 => "Shigatsu",
            5 => "Gogatsu",
            6 => "Rokugatsu",
            7 => "Shichigatsu",
            8 => "Hachigatsu",
            9 => "Kugatsu",
            10 => "Jugatsu",
            11 => "Juichigatsu",
            12 => "Junigatsu",
            _ => "",
        },
    }
}

pub fn format_localized_date(date: DateTime<Local>, format: &str, language_code: &str) -> String {
    if Language::from_config(language_code) == Language::En {
        return date.format(format).to_string();
    }

    // Use non-printable null-byte delimiters as sentinels: they can never appear
    // in any localized string, so each substitution is independent of the others.
    let template = format
        .replace("%A", "\x00WL\x00")
        .replace("%a", "\x00WS\x00")
        .replace("%B", "\x00ML\x00")
        .replace("%b", "\x00MS\x00");

    date.format(&template)
        .to_string()
        .replace("\x00WL\x00", weekday_long(date.weekday(), language_code))
        .replace("\x00WS\x00", weekday_short(date.weekday(), language_code))
        .replace("\x00ML\x00", month_long(date.month(), language_code))
        .replace("\x00MS\x00", month_short(date.month(), language_code))
}

#[cfg(test)]
mod tests {
    use super::{format_localized_date, translate, weekday_short, Language, TranslationKey};
    use chrono::{Local, TimeZone, Weekday};

    #[test]
    fn unknown_language_falls_back_to_english() {
        assert_eq!(Language::from_config("unknown"), Language::En);
        assert_eq!(translate(TranslationKey::Tomorrow, "unknown"), "Tomorrow");
    }

    #[test]
    fn returns_localized_weekday_abbreviations() {
        assert_eq!(weekday_short(Weekday::Mon, "fr"), "Lun");
        assert_eq!(weekday_short(Weekday::Tue, "de"), "Di");
        assert_eq!(weekday_short(Weekday::Sun, "ja"), "Nichi");
    }

    #[test]
    fn localizes_weekday_and_month_names_in_date_formats() {
        let date = Local.with_ymd_and_hms(2025, 10, 25, 12, 0, 0).unwrap();

        assert_eq!(
            format_localized_date(date, "%A, %d %B", "fr"),
            "Samedi, 25 Octobre"
        );
        assert_eq!(
            format_localized_date(date, "%a, %-d %b", "de"),
            "Sa, 25 Okt"
        );
    }
}
