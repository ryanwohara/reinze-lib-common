mod database;

use format_num::NumberFormat;
use mysql::{prelude::*, *};
use reqwest;
use std::cmp::Ord;

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f
            .to_uppercase()
            .chain(c.flat_map(|c| c.to_lowercase()))
            .collect(),
    }
}

// Gray
// c1
pub fn c1(s: &str) -> String {
    format!("\x0314{}", s)
}

// Red
// c2
pub fn c2(s: &str) -> String {
    format!("\x0304{}", s)
}

// Red
// c3
pub fn c3(s: &str) -> String {
    format!("\x0305{}", s)
}

// Green
// c4
pub fn c4(s: &str) -> String {
    format!("\x0303{}", s)
}

// Yellow
// c5
pub fn c5(s: &str) -> String {
    format!("\x0307{}", s)
}

// A function for wrapping a string in brackets that are colored gray
// l
pub fn l(s: &str) -> String {
    format!("{}{}{}", c1("["), c2(s), c1("]"))
}

// A function for wrapping a string in parenthesis that are colored gray
// p
pub fn p(s: &str) -> String {
    format!("{}{}{}", c1("("), c2(s), c1(")"))
}

// Converts a level to experience
pub fn level_to_xp(level: u32) -> u32 {
    let mut xp = 0;

    for i in 1..level {
        let x = i as f32;

        xp += (x + 300.0 * 2.0_f32.powf(x / 7.0)).floor() as u32 / 4;
    }

    xp
}

// Converts experience to a level
pub fn xp_to_level(xp: u32) -> u32 {
    for level in 1..=127 {
        if xp < level_to_xp(level) {
            return level - 1;
        }
    }

    0
}

// Adds commas to a number
pub fn commas(n: f64, f: &str) -> String {
    let num = NumberFormat::new();

    num.format(&format!(",{}", f), n)
}

// Adds commas to a string
pub fn commas_from_string(n: &str, f: &str) -> String {
    let n = match n.parse::<f64>() {
        Ok(n) => n,
        Err(_) => 0.0,
    };

    commas(n, f)
}

// Catches shorthand skill names and returns the full name
pub fn skill(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "overall" | "stats" | "total" => "Overall".to_string(),
        "attack" | "att" => "Attack".to_string(),
        "defence" | "def" => "Defence".to_string(),
        "strength" | "str" => "Strength".to_string(),
        "hitpoints" | "hp" => "Hitpoints".to_string(),
        "ranged" | "range" => "Ranged".to_string(),
        "prayer" | "pray" => "Prayer".to_string(),
        "magic" | "mage" => "Magic".to_string(),
        "cooking" | "cook" => "Cooking".to_string(),
        "woodcutting" | "wc" => "Woodcutting".to_string(),
        "fletching" | "fletch" => "Fletching".to_string(),
        "fishing" | "fish" => "Fishing".to_string(),
        "firemaking" | "fm" => "Firemaking".to_string(),
        "crafting" | "craft" => "Crafting".to_string(),
        "smithing" | "smith" => "Smithing".to_string(),
        "mining" | "mine" => "Mining".to_string(),
        "herblore" | "herb" => "Herblore".to_string(),
        "agility" | "agil" => "Agility".to_string(),
        "thieving" | "thief" => "Thieving".to_string(),
        "slayer" | "slay" => "Slayer".to_string(),
        "farming" | "farm" => "Farming".to_string(),
        "runecraft" | "rc" => "Runecraft".to_string(),
        "hunter" | "hunt" => "Hunter".to_string(),
        "construction" | "con" => "Construction".to_string(),
        _ => String::new(),
    }
}

// Returns a vector of all skills
pub fn skills() -> Vec<String> {
    vec![
        "Overall".to_string(),
        "Attack".to_string(),
        "Defence".to_string(),
        "Strength".to_string(),
        "Hitpoints".to_string(),
        "Ranged".to_string(),
        "Prayer".to_string(),
        "Magic".to_string(),
        "Cooking".to_string(),
        "Woodcutting".to_string(),
        "Fletching".to_string(),
        "Fishing".to_string(),
        "Firemaking".to_string(),
        "Crafting".to_string(),
        "Smithing".to_string(),
        "Mining".to_string(),
        "Herblore".to_string(),
        "Agility".to_string(),
        "Thieving".to_string(),
        "Slayer".to_string(),
        "Farming".to_string(),
        "Runecraft".to_string(),
        "Hunter".to_string(),
        "Construction".to_string(),
    ]
}

pub fn unranked(v: Vec<String>) -> String {
    if v.is_empty() {
        c2("Unranked")
    } else {
        v.join(&c1(" | "))
    }
}

pub fn not_found(v: Vec<String>) -> String {
    if v.is_empty() {
        c2("Not found")
    } else {
        v.join(&c1(" | "))
    }
}

pub fn get_cmb(
    att: &u32,
    str: &u32,
    def: &u32,
    hp: &u32,
    range: &u32,
    pray: &u32,
    mage: &u32,
) -> f64 {
    let base = ((def + hp) as f64 + ((pray + 0) as f64 / 2.0)) * 0.25;

    let melee = 0.325 * (att + str) as f64;
    let range = 0.325 * ((range.to_owned() as f64 / 2.0) + range.to_owned() as f64);
    let magic = 0.325 * ((mage.to_owned() as f64 / 2.0) + mage.to_owned() as f64);

    let max_contribution = f64::max(melee, f64::max(range, magic));

    base + max_contribution
}

pub fn get_rsn(author: &str, rsn_n: &str) -> core::result::Result<Vec<mysql::Row>, mysql::Error> {
    let mut conn = match database::connect() {
        Ok(conn) => conn,
        Err(e) => {
            println!("Error connecting to database: {}", e);
            return Err(e);
        }
    };

    let mut host = author.split("!").collect::<Vec<&str>>()[1];
    if host.starts_with("~") {
        host = host.split("~").collect::<Vec<&str>>()[1];
    }

    match conn.exec_first(
        "SELECT rsn FROM rsn WHERE host = :host AND rsn_ident = :rsn_n",
        params! { host, rsn_n },
    ) {
        Ok(Some(rsn)) => Ok(vec![rsn]),
        Ok(None) => Ok(vec![]),
        Err(e) => {
            println!("Error getting rsn: {}", e);
            Err(e)
        }
    }
}

pub fn get_stats(rsn: &str) -> core::result::Result<Vec<Vec<String>>, ()> {
    let mut stats = Vec::new();

    let body = match query_stats(rsn) {
        Ok(body) => body,
        Err(_) => return Err(()),
    };

    for line in body.lines() {
        let split = line
            .split(",")
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

        stats.push(split);
    }

    Ok(stats)
}

fn query_stats(rsn: &str) -> core::result::Result<String, ()> {
    let url = format!(
        "https://secure.runescape.com/m=hiscore_oldschool/index_lite.ws?player={}",
        rsn
    );

    let resp = match reqwest::blocking::get(&url) {
        Ok(resp) => resp,
        Err(e) => {
            println!("Error getting stats: {}", e);
            return Err(());
        }
    };

    let body = match resp.text() {
        Ok(body) => body.to_owned(),
        Err(e) => {
            println!("Error getting stats: {}", e);
            return Err(());
        }
    };

    Ok(body)
}

#[cfg(test)]
mod tests {
    // import names from outer (for mod tests) scope
    use super::*;

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("Hello"), "Hello");
        assert_eq!(capitalize("HELLO"), "Hello");
        assert_eq!(capitalize("hELLO"), "Hello");
        assert_eq!(capitalize("hElLo"), "Hello");
        assert_eq!(capitalize("123"), "123");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn test_c1() {
        assert_eq!(c1("hello"), "\x0314hello");
        assert_eq!(c1("Hello"), "\x0314Hello");
        assert_eq!(c1("HELLO"), "\x0314HELLO");
        assert_eq!(c1("hELLO"), "\x0314hELLO");
        assert_eq!(c1("hElLo"), "\x0314hElLo");
        assert_eq!(c1("123"), "\x0314123");
        assert_eq!(c1(""), "\x0314");
    }

    #[test]
    fn test_c2() {
        assert_eq!(c2("hello"), "\x0304hello");
        assert_eq!(c2("Hello"), "\x0304Hello");
        assert_eq!(c2("HELLO"), "\x0304HELLO");
        assert_eq!(c2("hELLO"), "\x0304hELLO");
        assert_eq!(c2("hElLo"), "\x0304hElLo");
        assert_eq!(c2("123"), "\x0304123");
        assert_eq!(c2(""), "\x0304");
    }

    #[test]
    fn test_c3() {
        assert_eq!(c3("hello"), "\x0305hello");
        assert_eq!(c3("Hello"), "\x0305Hello");
        assert_eq!(c3("HELLO"), "\x0305HELLO");
        assert_eq!(c3("hELLO"), "\x0305hELLO");
        assert_eq!(c3("hElLo"), "\x0305hElLo");
        assert_eq!(c3("123"), "\x0305123");
        assert_eq!(c3(""), "\x0305");
    }

    #[test]
    fn test_c4() {
        assert_eq!(c4("hello"), "\x0303hello");
        assert_eq!(c4("Hello"), "\x0303Hello");
        assert_eq!(c4("HELLO"), "\x0303HELLO");
        assert_eq!(c4("hELLO"), "\x0303hELLO");
        assert_eq!(c4("hElLo"), "\x0303hElLo");
        assert_eq!(c4("123"), "\x0303123");
        assert_eq!(c4(""), "\x0303");
    }

    #[test]
    fn test_c5() {
        assert_eq!(c5("hello"), "\x0307hello");
        assert_eq!(c5("Hello"), "\x0307Hello");
        assert_eq!(c5("HELLO"), "\x0307HELLO");
        assert_eq!(c5("hELLO"), "\x0307hELLO");
        assert_eq!(c5("hElLo"), "\x0307hElLo");
        assert_eq!(c5("123"), "\x0307123");
        assert_eq!(c5(""), "\x0307");
    }

    #[test]
    fn test_l() {
        assert_eq!(l("hello"), "\x0314[\x0304hello\x0314]");
        assert_eq!(l("Hello"), "\x0314[\x0304Hello\x0314]");
        assert_eq!(l("HELLO"), "\x0314[\x0304HELLO\x0314]");
        assert_eq!(l("hELLO"), "\x0314[\x0304hELLO\x0314]");
        assert_eq!(l("hElLo"), "\x0314[\x0304hElLo\x0314]");
        assert_eq!(l("123"), "\x0314[\x0304123\x0314]");
        assert_eq!(l(""), "\x0314[\x0304\x0314]");
    }

    #[test]
    fn test_p() {
        assert_eq!(p("hello"), "\x0314(\x0304hello\x0314)");
        assert_eq!(p("Hello"), "\x0314(\x0304Hello\x0314)");
        assert_eq!(p("HELLO"), "\x0314(\x0304HELLO\x0314)");
        assert_eq!(p("hELLO"), "\x0314(\x0304hELLO\x0314)");
        assert_eq!(p("hElLo"), "\x0314(\x0304hElLo\x0314)");
        assert_eq!(p("123"), "\x0314(\x0304123\x0314)");
        assert_eq!(p(""), "\x0314(\x0304\x0314)");
    }

    #[test]
    fn test_level_to_xp() {
        assert_eq!(level_to_xp(1), 0);
        assert_eq!(level_to_xp(2), 83);
        assert_eq!(level_to_xp(3), 174);
        assert_eq!(level_to_xp(4), 275);
        assert_eq!(level_to_xp(5), 387);
        assert_eq!(level_to_xp(6), 511);
        assert_eq!(level_to_xp(7), 648);
        assert_eq!(level_to_xp(8), 799);
        assert_eq!(level_to_xp(9), 966);
        assert_eq!(level_to_xp(10), 1151);
        assert_eq!(level_to_xp(11), 1355);
        assert_eq!(level_to_xp(12), 1580);
        assert_eq!(level_to_xp(13), 1829);
        assert_eq!(level_to_xp(14), 2103);
        assert_eq!(level_to_xp(15), 2406);
        assert_eq!(level_to_xp(16), 2740);
        assert_eq!(level_to_xp(17), 3109);
        assert_eq!(level_to_xp(18), 3517);
        assert_eq!(level_to_xp(19), 3967);
        assert_eq!(level_to_xp(20), 4463);
        assert_eq!(level_to_xp(21), 5011);
        assert_eq!(level_to_xp(22), 5616);
        assert_eq!(level_to_xp(23), 6283);
        assert_eq!(level_to_xp(24), 7020);
        assert_eq!(level_to_xp(25), 7833);
        assert_eq!(level_to_xp(26), 8730);
        assert_eq!(level_to_xp(27), 9720);
        assert_eq!(level_to_xp(28), 10813);
        assert_eq!(level_to_xp(29), 12020);
        assert_eq!(level_to_xp(30), 13352);
        assert_eq!(level_to_xp(45), 61495);
        assert_eq!(level_to_xp(55), 166614);
        assert_eq!(level_to_xp(75), 1210391);
        assert_eq!(level_to_xp(92), 6517217);
        assert_eq!(level_to_xp(95), 8771521);
        assert_eq!(level_to_xp(96), 9684539);
        assert_eq!(level_to_xp(97), 10692591);
        assert_eq!(level_to_xp(98), 11805568);
        assert_eq!(level_to_xp(99), 13034392);
    }
    #[test]
    fn test_xp_to_level() {
        assert_eq!(xp_to_level(0), 1);
        assert_eq!(xp_to_level(83), 2);
        assert_eq!(xp_to_level(174), 3);
        assert_eq!(xp_to_level(275), 4);
        assert_eq!(xp_to_level(387), 5);
        assert_eq!(xp_to_level(511), 6);
        assert_eq!(xp_to_level(648), 7);
        assert_eq!(xp_to_level(799), 8);
        assert_eq!(xp_to_level(966), 9);
        assert_eq!(xp_to_level(1151), 10);
        assert_eq!(xp_to_level(1355), 11);
        assert_eq!(xp_to_level(1580), 12);
        assert_eq!(xp_to_level(1829), 13);
        assert_eq!(xp_to_level(2103), 14);
        assert_eq!(xp_to_level(2406), 15);
        assert_eq!(xp_to_level(2740), 16);
        assert_eq!(xp_to_level(3109), 17);
        assert_eq!(xp_to_level(3517), 18);
        assert_eq!(xp_to_level(3967), 19);
        assert_eq!(xp_to_level(4463), 20);
        assert_eq!(xp_to_level(5011), 21);
        assert_eq!(xp_to_level(5616), 22);
        assert_eq!(xp_to_level(6283), 23);
        assert_eq!(xp_to_level(7020), 24);
        assert_eq!(xp_to_level(7833), 25);
        assert_eq!(xp_to_level(8730), 26);
        assert_eq!(xp_to_level(9720), 27);
        assert_eq!(xp_to_level(10813), 28);
        assert_eq!(xp_to_level(12020), 29);
        assert_eq!(xp_to_level(13352), 30);
        assert_eq!(xp_to_level(61495), 45);
        assert_eq!(xp_to_level(166614), 55);
        assert_eq!(xp_to_level(1210391), 75);
        assert_eq!(xp_to_level(6517217), 92);
        assert_eq!(xp_to_level(8771521), 95);
        assert_eq!(xp_to_level(9684539), 96);
        assert_eq!(xp_to_level(10692591), 97);
        assert_eq!(xp_to_level(11805568), 98);
        assert_eq!(xp_to_level(13034392), 99);
    }

    #[test]
    fn test_commas() {
        assert_eq!(commas(0.0, "d"), "0");
        assert_eq!(commas(1.0, "d"), "1");
        assert_eq!(commas(10.0, "d"), "10");
        assert_eq!(commas(100.0, "d"), "100");
        assert_eq!(commas(1000.0, "d"), "1,000");
        assert_eq!(commas(10000.0, "d"), "10,000");
        assert_eq!(commas(100000.0, "d"), "100,000");
        assert_eq!(commas(1000000.0, "d"), "1,000,000");
        assert_eq!(commas(10000000.0, "d"), "10,000,000");
        assert_eq!(commas(100000000.0, "d"), "100,000,000");
        assert_eq!(commas(1000000000.0, "d"), "1,000,000,000");
        assert_eq!(commas(10000000000.0, "d"), "10,000,000,000");
        assert_eq!(commas(100000000000.0, "d"), "100,000,000,000");
        assert_eq!(commas(1000000000000.0, "d"), "1,000,000,000,000");
        assert_eq!(commas(10000000000000.0, "d"), "10,000,000,000,000");
        assert_eq!(commas(100000000000000.0, "d"), "100,000,000,000,000");
        assert_eq!(commas(1000000000000000.0, "d"), "1,000,000,000,000,000");
        assert_eq!(commas(10000000000000000.0, "d"), "10,000,000,000,000,000");
        assert_eq!(commas(100000000000000000.0, "d"), "100,000,000,000,000,000");
        assert_eq!(
            commas(1000000000000000000.0, "d"),
            "1,000,000,000,000,000,000"
        );
        assert_eq!(
            commas(10000000000000000000.0, "d"),
            "10,000,000,000,000,000,000"
        );
        assert_eq!(
            commas(100000000000000000000.0, "d"),
            "100,000,000,000,000,000,000"
        );
    }

    #[test]
    fn test_commas_from_string() {
        assert_eq!(commas_from_string("0", "d"), "0");
        assert_eq!(commas_from_string("1", "d"), "1");
        assert_eq!(commas_from_string("10", "d"), "10");
        assert_eq!(commas_from_string("100", "d"), "100");
        assert_eq!(commas_from_string("1000", "d"), "1,000");
        assert_eq!(commas_from_string("10000", "d"), "10,000");
        assert_eq!(commas_from_string("100000", "d"), "100,000");
        assert_eq!(commas_from_string("1000000", "d"), "1,000,000");
        assert_eq!(commas_from_string("10000000", "d"), "10,000,000");
        assert_eq!(commas_from_string("100000000", "d"), "100,000,000");
        assert_eq!(commas_from_string("1000000000", "d"), "1,000,000,000");
        assert_eq!(commas_from_string("10000000000", "d"), "10,000,000,000");
        assert_eq!(commas_from_string("100000000000", "d"), "100,000,000,000");
        assert_eq!(
            commas_from_string("1000000000000", "d"),
            "1,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("10000000000000", "d"),
            "10,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("100000000000000", "d"),
            "100,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("1000000000000000", "d"),
            "1,000,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("10000000000000000", "d"),
            "10,000,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("100000000000000000", "d"),
            "100,000,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("1000000000000000000", "d"),
            "1,000,000,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("10000000000000000000", "d"),
            "10,000,000,000,000,000,000"
        );
        assert_eq!(
            commas_from_string("100000000000000000000", "d"),
            "100,000,000,000,000,000,000"
        );
    }

    #[test]
    fn test_skill() {
        assert_eq!(skill("overall"), "Overall");
        assert_eq!(skill("stats"), "Overall");
        assert_eq!(skill("total"), "Overall");
        assert_eq!(skill("attack"), "Attack");
        assert_eq!(skill("att"), "Attack");
        assert_eq!(skill("defence"), "Defence");
        assert_eq!(skill("def"), "Defence");
        assert_eq!(skill("strength"), "Strength");
        assert_eq!(skill("str"), "Strength");
        assert_eq!(skill("hitpoints"), "Hitpoints");
        assert_eq!(skill("hp"), "Hitpoints");
        assert_eq!(skill("ranged"), "Ranged");
        assert_eq!(skill("range"), "Ranged");
        assert_eq!(skill("prayer"), "Prayer");
        assert_eq!(skill("pray"), "Prayer");
        assert_eq!(skill("magic"), "Magic");
        assert_eq!(skill("mage"), "Magic");
        assert_eq!(skill("cooking"), "Cooking");
        assert_eq!(skill("cook"), "Cooking");
        assert_eq!(skill("woodcutting"), "Woodcutting");
        assert_eq!(skill("wc"), "Woodcutting");
        assert_eq!(skill("fletching"), "Fletching");
        assert_eq!(skill("fletch"), "Fletching");
        assert_eq!(skill("fishing"), "Fishing");
        assert_eq!(skill("fish"), "Fishing");
        assert_eq!(skill("firemaking"), "Firemaking");
        assert_eq!(skill("fm"), "Firemaking");
        assert_eq!(skill("crafting"), "Crafting");
        assert_eq!(skill("craft"), "Crafting");
        assert_eq!(skill("smithing"), "Smithing");
        assert_eq!(skill("smith"), "Smithing");
        assert_eq!(skill("mining"), "Mining");
        assert_eq!(skill("mine"), "Mining");
        assert_eq!(skill("herblore"), "Herblore");
        assert_eq!(skill("herb"), "Herblore");
        assert_eq!(skill("agility"), "Agility");
        assert_eq!(skill("agil"), "Agility");
        assert_eq!(skill("thieving"), "Thieving");
        assert_eq!(skill("thief"), "Thieving");
        assert_eq!(skill("slayer"), "Slayer");
        assert_eq!(skill("slay"), "Slayer");
        assert_eq!(skill("farming"), "Farming");
        assert_eq!(skill("farm"), "Farming");
        assert_eq!(skill("runecraft"), "Runecraft");
        assert_eq!(skill("rc"), "Runecraft");
        assert_eq!(skill("hunter"), "Hunter");
        assert_eq!(skill("hunt"), "Hunter");
        assert_eq!(skill("construction"), "Construction");
        assert_eq!(skill("con"), "Construction");
        assert_eq!(skill("invalid"), "");
    }

    #[test]
    fn test_skills() {
        assert_eq!(skills().len(), 24,);
        assert_eq!(
            skills(),
            vec![
                "Overall".to_string(),
                "Attack".to_string(),
                "Defence".to_string(),
                "Strength".to_string(),
                "Hitpoints".to_string(),
                "Ranged".to_string(),
                "Prayer".to_string(),
                "Magic".to_string(),
                "Cooking".to_string(),
                "Woodcutting".to_string(),
                "Fletching".to_string(),
                "Fishing".to_string(),
                "Firemaking".to_string(),
                "Crafting".to_string(),
                "Smithing".to_string(),
                "Mining".to_string(),
                "Herblore".to_string(),
                "Agility".to_string(),
                "Thieving".to_string(),
                "Slayer".to_string(),
                "Farming".to_string(),
                "Runecraft".to_string(),
                "Hunter".to_string(),
                "Construction".to_string(),
            ]
        );
    }
}
