use chrono::{DateTime, Datelike, TimeZone, Weekday};
use std::fmt::Display;
use strum_macros::EnumIter;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Language {
    #[default]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum TranslationKey {
    Feels,
    Like,
    Metric,
    Now,
    Max,
}

pub fn translate(key: TranslationKey, language: Language) -> &'static str {
    match (language, key) {
        (Language::En, TranslationKey::Feels) => "Feels",
        (Language::En, TranslationKey::Like) => "Like",
        (Language::En, TranslationKey::Metric) => "Metric",
        (Language::En, TranslationKey::Now) => "Now",
        (Language::En, TranslationKey::Max) => "Max",
        (Language::Fr, TranslationKey::Feels) => "Ress.",
        (Language::Fr, TranslationKey::Like) => "comme",
        (Language::Fr, TranslationKey::Metric) => "Mesure",
        (Language::Fr, TranslationKey::Now) => "Maint.",
        (Language::Fr, TranslationKey::Max) => "Max",
        (Language::De, TranslationKey::Feels) => "Gef.",
        (Language::De, TranslationKey::Like) => "wie",
        (Language::De, TranslationKey::Metric) => "Wert",
        (Language::De, TranslationKey::Now) => "Jetzt",
        (Language::De, TranslationKey::Max) => "Max",
        (Language::Es, TranslationKey::Feels) => "Se",
        (Language::Es, TranslationKey::Like) => "siente",
        (Language::Es, TranslationKey::Metric) => "Medida",
        (Language::Es, TranslationKey::Now) => "Ahora",
        (Language::Es, TranslationKey::Max) => "Max",
        (Language::Ja, TranslationKey::Feels) => "体感",
        (Language::Ja, TranslationKey::Like) => "温度",
        (Language::Ja, TranslationKey::Metric) => "指標",
        (Language::Ja, TranslationKey::Now) => "今",
        (Language::Ja, TranslationKey::Max) => "最大",
    }
}

pub fn weekday_short(weekday: Weekday, language: Language) -> &'static str {
    match language {
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
            Weekday::Mon => "月",
            Weekday::Tue => "火",
            Weekday::Wed => "水",
            Weekday::Thu => "木",
            Weekday::Fri => "金",
            Weekday::Sat => "土",
            Weekday::Sun => "日",
        },
    }
}

pub fn weekday_long(weekday: Weekday, language: Language) -> &'static str {
    match language {
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
            Weekday::Mon => "月曜日",
            Weekday::Tue => "火曜日",
            Weekday::Wed => "水曜日",
            Weekday::Thu => "木曜日",
            Weekday::Fri => "金曜日",
            Weekday::Sat => "土曜日",
            Weekday::Sun => "日曜日",
        },
    }
}

pub fn month_short(month: u32, language: Language) -> &'static str {
    match language {
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
            1 => "1月",
            2 => "2月",
            3 => "3月",
            4 => "4月",
            5 => "5月",
            6 => "6月",
            7 => "7月",
            8 => "8月",
            9 => "9月",
            10 => "10月",
            11 => "11月",
            12 => "12月",
            _ => "",
        },
    }
}

pub fn month_long(month: u32, language: Language) -> &'static str {
    match language {
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
            12 => "Décembre",
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
            1 => "1月",
            2 => "2月",
            3 => "3月",
            4 => "4月",
            5 => "5月",
            6 => "6月",
            7 => "7月",
            8 => "8月",
            9 => "9月",
            10 => "10月",
            11 => "11月",
            12 => "12月",
            _ => "",
        },
    }
}

/// Swaps chrono's `%A`/`%a`/`%B`/`%b` specifiers for null-byte-delimited
/// sentinels (`format_localized_date` replaces these with translated names
/// after chrono has rendered everything else), leaving every other
/// specifier — including an escaped `%%` — untouched.
///
/// This walks the string one `%`-token at a time instead of doing a plain
/// substring replace, specifically so `%%A` (chrono's escape for a literal
/// "%A") isn't misread as the specifier `%A`: a substring replace would
/// match the `%A` inside `%%A` and corrupt the escape into an invalid
/// specifier, which made chrono's formatter — and `.to_string()` on it —
/// panic.
fn substitute_localizable_specifiers(format: &str) -> String {
    let mut result = String::with_capacity(format.len());
    let mut chars = format.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '%' {
            result.push(c);
            continue;
        }
        match chars.peek() {
            Some('%') => {
                // Escaped literal percent — leave both chars for chrono.
                result.push('%');
                result.push(chars.next().expect("peek confirmed a next char"));
            }
            Some('A') => {
                result.push_str("\x00WL\x00");
                chars.next();
            }
            Some('a') => {
                result.push_str("\x00WS\x00");
                chars.next();
            }
            Some('B') => {
                result.push_str("\x00ML\x00");
                chars.next();
            }
            Some('b') => {
                result.push_str("\x00MS\x00");
                chars.next();
            }
            // Any other specifier (or a trailing '%'): leave the '%' for
            // chrono to interpret together with whatever follows.
            _ => result.push('%'),
        }
    }

    result
}

pub fn format_localized_date<Tz>(date: DateTime<Tz>, format: &str, language: Language) -> String
where
    Tz: TimeZone,
    Tz::Offset: Display,
{
    if language == Language::En {
        return date.format(format).to_string();
    }

    let template = substitute_localizable_specifiers(format);

    date.format(&template)
        .to_string()
        .replace("\x00WL\x00", weekday_long(date.weekday(), language))
        .replace("\x00WS\x00", weekday_short(date.weekday(), language))
        .replace("\x00ML\x00", month_long(date.month(), language))
        .replace("\x00MS\x00", month_short(date.month(), language))
}

#[cfg(test)]
mod tests {
    use super::{
        format_localized_date, translate, weekday_long, weekday_short, Language, TranslationKey,
    };
    use chrono::{Local, TimeZone, Weekday};

    #[test]
    fn unknown_language_falls_back_to_english() {
        assert_eq!(Language::from_config("unknown"), Language::En);
        assert_eq!(translate(TranslationKey::Feels, Language::En), "Feels");
    }

    #[test]
    fn returns_localized_weekday_abbreviations() {
        assert_eq!(weekday_short(Weekday::Mon, Language::Fr), "Lun");
        assert_eq!(weekday_short(Weekday::Tue, Language::De), "Di");
        assert_eq!(weekday_short(Weekday::Sun, Language::Ja), "日");
    }

    #[test]
    fn localizes_weekday_and_month_names_in_date_formats() {
        let date = Local.with_ymd_and_hms(2025, 10, 25, 12, 0, 0).unwrap();

        assert_eq!(
            format_localized_date(date, "%A, %d %B", Language::Fr),
            "Samedi, 25 Octobre"
        );
        assert_eq!(
            format_localized_date(date, "%a, %-d %b", Language::De),
            "Sa, 25 Okt"
        );
    }

    #[test]
    fn escaped_percent_before_a_specifier_letter_is_not_corrupted() {
        // "%%A" is chrono's escape for a literal "%A", not the weekday
        // specifier — a naive substring replace of "%A" would match inside
        // it and corrupt the escape into an invalid format, which used to
        // make chrono's formatter (and `.to_string()` on it) panic.
        let date = Local.with_ymd_and_hms(2025, 10, 25, 12, 0, 0).unwrap();
        assert_eq!(
            format_localized_date(date, "%%A, %d %B", Language::Fr),
            "%A, 25 Octobre"
        );
    }

    #[test]
    fn weekday_long_names_fit_the_rotated_chart_label_budget() {
        // draw_tomorrow_line rotates weekday_long into a fixed-height chart
        // slot for Latin-script languages. All weekdays within a language
        // must use the same (long) form, so this asserts the budget for
        // every day rather than falling back per-day at render time, which
        // would produce visually inconsistent label shapes within a locale.
        const MAX_LEN: usize = 10;
        for language in [Language::En, Language::Fr, Language::De, Language::Es] {
            for weekday in [
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ] {
                let name = weekday_long(weekday, language);
                assert!(
                    name.len() <= MAX_LEN,
                    "{name:?} ({language:?}, {weekday:?}) is {} bytes, over the {MAX_LEN}-byte rotated label budget",
                    name.len()
                );
            }
        }
    }
}
