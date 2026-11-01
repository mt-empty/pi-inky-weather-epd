use chrono::{DateTime, Datelike, TimeZone, Weekday};
use serde::Deserialize;
use std::fmt::Display;
use strum_macros::{Display as StrumDisplay, EnumIter};

/// A supported UI language (`render_options.language`), deserialized
/// directly from config
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter, Deserialize, StrumDisplay)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Language {
    #[default]
    En,
    Fr,
    De,
    Es,
    Ja,
    Ru,
    Uk,
    It,
    Pt,
    Nl,
    Pl,
    Sv,
    #[serde(alias = "no")]
    Nb,
    Da,
    Cs,
}

impl Language {
    /// Whether this language's native clock convention is 12-hour or 24-hour.
    pub fn uses_twelve_hour_clock(self) -> bool {
        match self {
            Self::En => true,
            Self::Fr
            | Self::De
            | Self::Es
            | Self::Ja
            | Self::Ru
            | Self::Uk
            | Self::It
            | Self::Pt
            | Self::Nl
            | Self::Pl
            | Self::Sv
            | Self::Nb
            | Self::Da
            | Self::Cs => false,
        }
    }

    /// The header date format used when `render_options.date_format` is not
    /// set. Each follows the language's own word order and punctuation; for
    /// ru/uk/pl/cs, `%B` is the genitive month form, which is only correct
    /// directly after a day number.
    pub fn default_date_format(self) -> &'static str {
        match self {
            Self::En => "%A, %d %B",
            Self::Fr | Self::It | Self::Nl | Self::Sv => "%A %-d %B",
            Self::De => "%A, %-d. %B",
            Self::Es | Self::Pt => "%A, %-d de %B",
            Self::Ja => "%B%-d日 (%a)",
            Self::Ru | Self::Uk | Self::Pl => "%A, %-d %B",
            Self::Nb | Self::Da | Self::Cs => "%A %-d. %B",
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
        (Language::Fr, TranslationKey::Feels) => "Ress",
        (Language::Fr, TranslationKey::Like) => "comme",
        (Language::Fr, TranslationKey::Metric) => "Mesure",
        (Language::Fr, TranslationKey::Now) => "Maint",
        (Language::Fr, TranslationKey::Max) => "Max",
        (Language::De, TranslationKey::Feels) => "Gef",
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
        (Language::Ru, TranslationKey::Feels) => "Ощущ",
        (Language::Ru, TranslationKey::Like) => "как",
        (Language::Ru, TranslationKey::Metric) => "Знач",
        (Language::Ru, TranslationKey::Now) => "Сейч",
        (Language::Ru, TranslationKey::Max) => "Макс",
        (Language::Uk, TranslationKey::Feels) => "Відч",
        (Language::Uk, TranslationKey::Like) => "як",
        (Language::Uk, TranslationKey::Metric) => "Знач",
        (Language::Uk, TranslationKey::Now) => "Нині",
        (Language::Uk, TranslationKey::Max) => "Макс",
        (Language::It, TranslationKey::Feels) => "Perc",
        (Language::It, TranslationKey::Like) => "come",
        (Language::It, TranslationKey::Metric) => "Valore",
        (Language::It, TranslationKey::Now) => "Ora",
        (Language::It, TranslationKey::Max) => "Max",
        (Language::Pt, TranslationKey::Feels) => "Sens",
        (Language::Pt, TranslationKey::Like) => "como",
        (Language::Pt, TranslationKey::Metric) => "Valor",
        (Language::Pt, TranslationKey::Now) => "Agora",
        (Language::Pt, TranslationKey::Max) => "Máx",
        (Language::Nl, TranslationKey::Feels) => "Voelt",
        (Language::Nl, TranslationKey::Like) => "als",
        (Language::Nl, TranslationKey::Metric) => "Maat",
        (Language::Nl, TranslationKey::Now) => "Nu",
        (Language::Nl, TranslationKey::Max) => "Max",
        (Language::Pl, TranslationKey::Feels) => "Odcz",
        (Language::Pl, TranslationKey::Like) => "jak",
        (Language::Pl, TranslationKey::Metric) => "Wart",
        (Language::Pl, TranslationKey::Now) => "Teraz",
        (Language::Pl, TranslationKey::Max) => "Maks",
        (Language::Sv, TranslationKey::Feels) => "Känns",
        (Language::Sv, TranslationKey::Like) => "som",
        (Language::Sv, TranslationKey::Metric) => "Värde",
        (Language::Sv, TranslationKey::Now) => "Nu",
        (Language::Sv, TranslationKey::Max) => "Max",
        (Language::Nb, TranslationKey::Feels) => "Føles",
        (Language::Nb, TranslationKey::Like) => "som",
        (Language::Nb, TranslationKey::Metric) => "Verdi",
        (Language::Nb, TranslationKey::Now) => "Nå",
        (Language::Nb, TranslationKey::Max) => "Maks",
        (Language::Da, TranslationKey::Feels) => "Føles",
        (Language::Da, TranslationKey::Like) => "som",
        (Language::Da, TranslationKey::Metric) => "Værdi",
        (Language::Da, TranslationKey::Now) => "Nu",
        (Language::Da, TranslationKey::Max) => "Maks",
        (Language::Cs, TranslationKey::Feels) => "Pocit",
        (Language::Cs, TranslationKey::Like) => "jako",
        (Language::Cs, TranslationKey::Metric) => "Hodn",
        (Language::Cs, TranslationKey::Now) => "Nyní",
        (Language::Cs, TranslationKey::Max) => "Max",
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
        Language::Ru => match weekday {
            Weekday::Mon => "Пн",
            Weekday::Tue => "Вт",
            Weekday::Wed => "Ср",
            Weekday::Thu => "Чт",
            Weekday::Fri => "Пт",
            Weekday::Sat => "Сб",
            Weekday::Sun => "Вс",
        },
        Language::Uk => match weekday {
            Weekday::Mon => "Пн",
            Weekday::Tue => "Вт",
            Weekday::Wed => "Ср",
            Weekday::Thu => "Чт",
            Weekday::Fri => "Пт",
            Weekday::Sat => "Сб",
            Weekday::Sun => "Нд",
        },
        Language::It => match weekday {
            Weekday::Mon => "Lun",
            Weekday::Tue => "Mar",
            Weekday::Wed => "Mer",
            Weekday::Thu => "Gio",
            Weekday::Fri => "Ven",
            Weekday::Sat => "Sab",
            Weekday::Sun => "Dom",
        },
        Language::Pt => match weekday {
            Weekday::Mon => "Seg",
            Weekday::Tue => "Ter",
            Weekday::Wed => "Qua",
            Weekday::Thu => "Qui",
            Weekday::Fri => "Sex",
            Weekday::Sat => "Sáb",
            Weekday::Sun => "Dom",
        },
        Language::Nl => match weekday {
            Weekday::Mon => "Ma",
            Weekday::Tue => "Di",
            Weekday::Wed => "Wo",
            Weekday::Thu => "Do",
            Weekday::Fri => "Vr",
            Weekday::Sat => "Za",
            Weekday::Sun => "Zo",
        },
        Language::Pl => match weekday {
            Weekday::Mon => "Pn",
            Weekday::Tue => "Wt",
            Weekday::Wed => "Śr",
            Weekday::Thu => "Cz",
            Weekday::Fri => "Pt",
            Weekday::Sat => "So",
            Weekday::Sun => "Nd",
        },
        Language::Sv => match weekday {
            Weekday::Mon => "Mån",
            Weekday::Tue => "Tis",
            Weekday::Wed => "Ons",
            Weekday::Thu => "Tor",
            Weekday::Fri => "Fre",
            Weekday::Sat => "Lör",
            Weekday::Sun => "Sön",
        },
        Language::Nb => match weekday {
            Weekday::Mon => "Man",
            Weekday::Tue => "Tir",
            Weekday::Wed => "Ons",
            Weekday::Thu => "Tor",
            Weekday::Fri => "Fre",
            Weekday::Sat => "Lør",
            Weekday::Sun => "Søn",
        },
        Language::Da => match weekday {
            Weekday::Mon => "Man",
            Weekday::Tue => "Tir",
            Weekday::Wed => "Ons",
            Weekday::Thu => "Tor",
            Weekday::Fri => "Fre",
            Weekday::Sat => "Lør",
            Weekday::Sun => "Søn",
        },
        Language::Cs => match weekday {
            Weekday::Mon => "Po",
            Weekday::Tue => "Út",
            Weekday::Wed => "St",
            Weekday::Thu => "Čt",
            Weekday::Fri => "Pá",
            Weekday::Sat => "So",
            Weekday::Sun => "Ne",
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
        Language::Ru => match weekday {
            Weekday::Mon => "Понедельник",
            Weekday::Tue => "Вторник",
            Weekday::Wed => "Среда",
            Weekday::Thu => "Четверг",
            Weekday::Fri => "Пятница",
            Weekday::Sat => "Суббота",
            Weekday::Sun => "Воскресенье",
        },
        Language::Uk => match weekday {
            Weekday::Mon => "Понеділок",
            Weekday::Tue => "Вівторок",
            Weekday::Wed => "Середа",
            Weekday::Thu => "Четвер",
            Weekday::Fri => "П’ятниця",
            Weekday::Sat => "Субота",
            Weekday::Sun => "Неділя",
        },
        Language::It => match weekday {
            Weekday::Mon => "Lunedì",
            Weekday::Tue => "Martedì",
            Weekday::Wed => "Mercoledì",
            Weekday::Thu => "Giovedì",
            Weekday::Fri => "Venerdì",
            Weekday::Sat => "Sabato",
            Weekday::Sun => "Domenica",
        },
        Language::Pt => match weekday {
            Weekday::Mon => "Segunda",
            Weekday::Tue => "Terça",
            Weekday::Wed => "Quarta",
            Weekday::Thu => "Quinta",
            Weekday::Fri => "Sexta",
            Weekday::Sat => "Sábado",
            Weekday::Sun => "Domingo",
        },
        Language::Nl => match weekday {
            Weekday::Mon => "Maandag",
            Weekday::Tue => "Dinsdag",
            Weekday::Wed => "Woensdag",
            Weekday::Thu => "Donderdag",
            Weekday::Fri => "Vrijdag",
            Weekday::Sat => "Zaterdag",
            Weekday::Sun => "Zondag",
        },
        Language::Pl => match weekday {
            Weekday::Mon => "Poniedziałek",
            Weekday::Tue => "Wtorek",
            Weekday::Wed => "Środa",
            Weekday::Thu => "Czwartek",
            Weekday::Fri => "Piątek",
            Weekday::Sat => "Sobota",
            Weekday::Sun => "Niedziela",
        },
        Language::Sv => match weekday {
            Weekday::Mon => "Måndag",
            Weekday::Tue => "Tisdag",
            Weekday::Wed => "Onsdag",
            Weekday::Thu => "Torsdag",
            Weekday::Fri => "Fredag",
            Weekday::Sat => "Lördag",
            Weekday::Sun => "Söndag",
        },
        Language::Nb => match weekday {
            Weekday::Mon => "Mandag",
            Weekday::Tue => "Tirsdag",
            Weekday::Wed => "Onsdag",
            Weekday::Thu => "Torsdag",
            Weekday::Fri => "Fredag",
            Weekday::Sat => "Lørdag",
            Weekday::Sun => "Søndag",
        },
        Language::Da => match weekday {
            Weekday::Mon => "Mandag",
            Weekday::Tue => "Tirsdag",
            Weekday::Wed => "Onsdag",
            Weekday::Thu => "Torsdag",
            Weekday::Fri => "Fredag",
            Weekday::Sat => "Lørdag",
            Weekday::Sun => "Søndag",
        },
        Language::Cs => match weekday {
            Weekday::Mon => "Pondělí",
            Weekday::Tue => "Úterý",
            Weekday::Wed => "Středa",
            Weekday::Thu => "Čtvrtek",
            Weekday::Fri => "Pátek",
            Weekday::Sat => "Sobota",
            Weekday::Sun => "Neděle",
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
            1 => "janv",
            2 => "févr",
            3 => "mars",
            4 => "avr",
            5 => "mai",
            6 => "juin",
            7 => "juil",
            8 => "août",
            9 => "sept",
            10 => "oct",
            11 => "nov",
            12 => "déc",
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
            1 => "ene",
            2 => "feb",
            3 => "mar",
            4 => "abr",
            5 => "may",
            6 => "jun",
            7 => "jul",
            8 => "ago",
            9 => "sep",
            10 => "oct",
            11 => "nov",
            12 => "dic",
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
        Language::Ru => match month {
            1 => "янв",
            2 => "фев",
            3 => "мар",
            4 => "апр",
            5 => "мая",
            6 => "июн",
            7 => "июл",
            8 => "авг",
            9 => "сен",
            10 => "окт",
            11 => "ноя",
            12 => "дек",
            _ => "",
        },
        Language::Uk => match month {
            1 => "січ",
            2 => "лют",
            3 => "бер",
            4 => "кві",
            5 => "тра",
            6 => "чер",
            7 => "лип",
            8 => "сер",
            9 => "вер",
            10 => "жов",
            11 => "лис",
            12 => "гру",
            _ => "",
        },
        Language::It => match month {
            1 => "gen",
            2 => "feb",
            3 => "mar",
            4 => "apr",
            5 => "mag",
            6 => "giu",
            7 => "lug",
            8 => "ago",
            9 => "set",
            10 => "ott",
            11 => "nov",
            12 => "dic",
            _ => "",
        },
        Language::Pt => match month {
            1 => "jan",
            2 => "fev",
            3 => "mar",
            4 => "abr",
            5 => "mai",
            6 => "jun",
            7 => "jul",
            8 => "ago",
            9 => "set",
            10 => "out",
            11 => "nov",
            12 => "dez",
            _ => "",
        },
        Language::Nl => match month {
            1 => "jan",
            2 => "feb",
            3 => "mrt",
            4 => "apr",
            5 => "mei",
            6 => "jun",
            7 => "jul",
            8 => "aug",
            9 => "sep",
            10 => "okt",
            11 => "nov",
            12 => "dec",
            _ => "",
        },
        Language::Pl => match month {
            1 => "sty",
            2 => "lut",
            3 => "mar",
            4 => "kwi",
            5 => "maj",
            6 => "cze",
            7 => "lip",
            8 => "sie",
            9 => "wrz",
            10 => "paź",
            11 => "lis",
            12 => "gru",
            _ => "",
        },
        Language::Sv => match month {
            1 => "jan",
            2 => "feb",
            3 => "mar",
            4 => "apr",
            5 => "maj",
            6 => "jun",
            7 => "jul",
            8 => "aug",
            9 => "sep",
            10 => "okt",
            11 => "nov",
            12 => "dec",
            _ => "",
        },
        Language::Nb => match month {
            1 => "jan",
            2 => "feb",
            3 => "mar",
            4 => "apr",
            5 => "mai",
            6 => "jun",
            7 => "jul",
            8 => "aug",
            9 => "sep",
            10 => "okt",
            11 => "nov",
            12 => "des",
            _ => "",
        },
        Language::Da => match month {
            1 => "jan",
            2 => "feb",
            3 => "mar",
            4 => "apr",
            5 => "maj",
            6 => "jun",
            7 => "jul",
            8 => "aug",
            9 => "sep",
            10 => "okt",
            11 => "nov",
            12 => "dec",
            _ => "",
        },
        Language::Cs => match month {
            1 => "led",
            2 => "úno",
            3 => "bře",
            4 => "dub",
            5 => "kvě",
            6 => "čvn",
            7 => "čvc",
            8 => "srp",
            9 => "zář",
            10 => "říj",
            11 => "lis",
            12 => "pro",
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
            1 => "janvier",
            2 => "février",
            3 => "mars",
            4 => "avril",
            5 => "mai",
            6 => "juin",
            7 => "juillet",
            8 => "août",
            9 => "septembre",
            10 => "octobre",
            11 => "novembre",
            12 => "décembre",
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
            1 => "enero",
            2 => "febrero",
            3 => "marzo",
            4 => "abril",
            5 => "mayo",
            6 => "junio",
            7 => "julio",
            8 => "agosto",
            9 => "septiembre",
            10 => "octubre",
            11 => "noviembre",
            12 => "diciembre",
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
        Language::Ru => match month {
            1 => "января",
            2 => "февраля",
            3 => "марта",
            4 => "апреля",
            5 => "мая",
            6 => "июня",
            7 => "июля",
            8 => "августа",
            9 => "сентября",
            10 => "октября",
            11 => "ноября",
            12 => "декабря",
            _ => "",
        },
        Language::Uk => match month {
            1 => "січня",
            2 => "лютого",
            3 => "березня",
            4 => "квітня",
            5 => "травня",
            6 => "червня",
            7 => "липня",
            8 => "серпня",
            9 => "вересня",
            10 => "жовтня",
            11 => "листопада",
            12 => "грудня",
            _ => "",
        },
        Language::It => match month {
            1 => "gennaio",
            2 => "febbraio",
            3 => "marzo",
            4 => "aprile",
            5 => "maggio",
            6 => "giugno",
            7 => "luglio",
            8 => "agosto",
            9 => "settembre",
            10 => "ottobre",
            11 => "novembre",
            12 => "dicembre",
            _ => "",
        },
        Language::Pt => match month {
            1 => "janeiro",
            2 => "fevereiro",
            3 => "março",
            4 => "abril",
            5 => "maio",
            6 => "junho",
            7 => "julho",
            8 => "agosto",
            9 => "setembro",
            10 => "outubro",
            11 => "novembro",
            12 => "dezembro",
            _ => "",
        },
        Language::Nl => match month {
            1 => "januari",
            2 => "februari",
            3 => "maart",
            4 => "april",
            5 => "mei",
            6 => "juni",
            7 => "juli",
            8 => "augustus",
            9 => "september",
            10 => "oktober",
            11 => "november",
            12 => "december",
            _ => "",
        },
        Language::Pl => match month {
            1 => "stycznia",
            2 => "lutego",
            3 => "marca",
            4 => "kwietnia",
            5 => "maja",
            6 => "czerwca",
            7 => "lipca",
            8 => "sierpnia",
            9 => "września",
            10 => "października",
            11 => "listopada",
            12 => "grudnia",
            _ => "",
        },
        Language::Sv => match month {
            1 => "januari",
            2 => "februari",
            3 => "mars",
            4 => "april",
            5 => "maj",
            6 => "juni",
            7 => "juli",
            8 => "augusti",
            9 => "september",
            10 => "oktober",
            11 => "november",
            12 => "december",
            _ => "",
        },
        Language::Nb => match month {
            1 => "januar",
            2 => "februar",
            3 => "mars",
            4 => "april",
            5 => "mai",
            6 => "juni",
            7 => "juli",
            8 => "august",
            9 => "september",
            10 => "oktober",
            11 => "november",
            12 => "desember",
            _ => "",
        },
        Language::Da => match month {
            1 => "januar",
            2 => "februar",
            3 => "marts",
            4 => "april",
            5 => "maj",
            6 => "juni",
            7 => "juli",
            8 => "august",
            9 => "september",
            10 => "oktober",
            11 => "november",
            12 => "december",
            _ => "",
        },
        Language::Cs => match month {
            1 => "ledna",
            2 => "února",
            3 => "března",
            4 => "dubna",
            5 => "května",
            6 => "června",
            7 => "července",
            8 => "srpna",
            9 => "září",
            10 => "října",
            11 => "listopadu",
            12 => "prosince",
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
    use super::{format_localized_date, weekday_short, Language};
    use chrono::{Local, TimeZone, Weekday};
    use strum::IntoEnumIterator;

    #[test]
    fn every_language_round_trips_through_its_config_code() {
        for language in Language::iter() {
            let code = language.to_string();
            assert_eq!(code.len(), 2, "{language:?} code {code:?} is not ISO 639-1");
            assert_eq!(
                serde_json::from_str::<Language>(&format!("{code:?}")).unwrap(),
                language
            );
        }
    }

    #[test]
    fn default_date_formats_render_grammatically() {
        let date = Local.with_ymd_and_hms(2025, 10, 25, 12, 0, 0).unwrap();
        for (language, expected) in [
            (Language::En, "Saturday, 25 October"),
            (Language::De, "Samstag, 25. Oktober"),
            (Language::Es, "Sábado, 25 de octubre"),
            (Language::Ja, "10月25日 (土)"),
            (Language::Pt, "Sábado, 25 de outubro"),
            (Language::Pl, "Sobota, 25 października"),
            (Language::Cs, "Sobota 25. října"),
        ] {
            assert_eq!(
                format_localized_date(date, language.default_date_format(), language),
                expected
            );
        }
    }

    #[test]
    fn norwegian_accepts_both_nb_and_no() {
        assert_eq!(
            serde_json::from_str::<Language>("\"no\"").unwrap(),
            Language::Nb
        );
    }

    #[test]
    fn rejects_unsupported_language_code() {
        assert!(serde_json::from_str::<Language>("\"unknown\"").is_err());
    }

    #[test]
    fn returns_localized_weekday_abbreviations() {
        assert_eq!(weekday_short(Weekday::Mon, Language::Fr), "Lun");
        assert_eq!(weekday_short(Weekday::Tue, Language::De), "Di");
        assert_eq!(weekday_short(Weekday::Sun, Language::Ja), "日");
        assert_eq!(weekday_short(Weekday::Sat, Language::Ru), "Сб");
    }

    #[test]
    fn localizes_weekday_and_month_names_in_date_formats() {
        let date = Local.with_ymd_and_hms(2025, 10, 25, 12, 0, 0).unwrap();

        assert_eq!(
            format_localized_date(date, "%A, %d %B", Language::Fr),
            "Samedi, 25 octobre"
        );
        assert_eq!(
            format_localized_date(date, "%a, %-d %b", Language::De),
            "Sa, 25 Okt"
        );
        // Russian months take the genitive case after a day number.
        assert_eq!(
            format_localized_date(date, "%A, %d %B", Language::Ru),
            "Суббота, 25 октября"
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
            "%A, 25 octobre"
        );
    }
}
