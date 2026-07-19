/// # Integration tests for free_surface::config::dicofile
mod test_helpers;

use free_surface::config::configvalue::{ConfigValue, DicoType};
use free_surface::config::dicofile::parse_file;

use test_helpers::{fixture, read_lines, telemac_file};

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn telemac2d_dico() {
    let dico_path =
        telemac_file("sources/telemac2d/telemac2d.dico").expect("Can't get telemac file");
    let dico = parse_file(dico_path).expect("dico is ok");

    // check that we parsed all the keywords (and no extra keyword!)
    assert_eq!(dico.len(), 376);

    // Check we got all expected keywords
    let keywords_names_file = fixture("telemac2d_keyword_names.txt");
    let lines = read_lines(keywords_names_file)
        .expect("can't read lines of telemac2d_keyword_names.txt fixture");

    for line in lines.map_while(Result::ok) {
        let keyword_name = line.trim();

        if keyword_name.is_empty() {
            continue;
        }

        assert!(
            dico.get(keyword_name).is_some(),
            "dico has no {keyword_name}"
        );
    }

    let title_keyword = dico
        .get("title")
        .expect("no 'title' keyword in telemac2d dico");
    assert_eq!(
        title_keyword.default(),
        ConfigValue::String(String::from(""))
    );
    assert_eq!(title_keyword.type_, DicoType::String);
    assert_eq!(title_keyword.nargs, 1);
    assert_eq!(title_keyword.level, 0);
}
