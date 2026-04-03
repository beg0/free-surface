use super::super::parse_helpers::DamoclesError;
use super::dicofile::parse_dico;
use super::dicofile::Dico;
use super::*;

const MY_FILE: &str = "/dave/null";

fn make_dico() -> Dico {
    let dico_content = indoc::indoc! {"
        NOM = 'QUI MAMAN AIME'
        NOM1 = 'WHO MOM LOVES'
        TYPE = STRING
        MNEMO = WML
        INDEX = 1
        NIVEAU = 2
        /
        NOM = 'MON AGE'
        NOM1 = 'MY AGE'
        TYPE = INTEGER
        CONTROLE = 0; 99
        MNEMO = MA
        INDEX = 2
        NIVEAU = 2
        /
        NOM = 'AGE DES ENFANTS'
        NOM1 = 'AGE OF CHILDREN'
        TYPE = INTEGER
        CONTROLE = 0; 99
        MNEMO = LADE
        INDEX = 3
        NIVEAU = 2
        TAILLE = 5
        /
        NOM = 'TAILLE DES ENFANTS'
        NOM1 = 'SIZE OF CHILDREN'
        TYPE = REAL
        CONTROLE = 0.20;2.50
        MNEMO = TDE
        NIVEAU = 2
        INDEX = 4
        TAILLE = 5
        /
        NOM = 'NOMS DES ENFANTS'
        NOM1 = 'NAMES OF CHILDREN'
        TYPE = STRING
        MNEMO = NDE
        NIVEAU = 2
        INDEX = 5
        TAILLE = 5
        /
        NOM = 'SEXE DES ENFANTS'
        NOM1 = 'SEX OF CHILDREN'
        TYPE = STRING
        CHOIX =
        'GARCON';
        'FILLE'
        CHOIX1 = 'BOY';'GIRL'
        MNEMO = SDE
        NIVEAU = 2
        INDEX = 6
        TAILLE = 5
        /
        NOM = 'SUIS JE SERIEUX'
        NOM1 = 'AM I SERIOUS'
        TYPE = LOGICAL
        MNEMO = AIS
        INDEX = 7
        NIVEAU = 2
        /
        NOM = 'MON NOMBRE FAVORIS'
        NOM1 = 'MY FAVORITE NUMBER'
        TYPE = REAL
        MNEMO = MFN
        INDEX = 8
        NIVEAU = 2
        /
        NOM = 'COMPTE EN BANK'
        NOM1 = 'BANK ACCOUNT'
        TYPE = INTEGER
        MNEMO = BA
        INDEX = 9
        NIVEAU = 2
    "};
    parse_dico(dico_content, MY_FILE).expect("Can't create dico")
}

// --- Helpers ---

fn parse_ok(input: &str) -> HashMap<String, ConfigValue> {
    let dico = make_dico();
    let parser = Parser::new(&dico);
    parser.parse(input).expect("Expected successful parse")
}

fn parse_err(input: &str) -> Vec<Box<dyn std::error::Error>> {
    let dico = make_dico();
    let parser = Parser::new(&dico);
    parser.parse(input).expect_err("Expected parse errors")
}

// --- Happy path ---

#[test]
fn test_string_unquoted() {
    let config = parse_ok("who mom loves = me");
    assert_eq!(config["WHO MOM LOVES"], ConfigValue::String("me".into()));
}

#[test]
fn test_string_quoted() {
    let config = parse_ok(r#"who mom loves = 'definitely me'"#);
    assert_eq!(
        config["WHO MOM LOVES"],
        ConfigValue::String("definitely me".into())
    );
}

#[test]
fn test_integer() {
    let config = parse_ok("my age = 25");
    assert_eq!(config["MY AGE"], ConfigValue::Integer(25));
}

#[test]
fn test_negative_integer() {
    let config = parse_ok("bank account = -1000");
    assert_eq!(config["BANK ACCOUNT"], ConfigValue::Integer(-1000));
}

#[test]
fn test_float() {
    let config = parse_ok("my favorite number = 3.14");
    assert_eq!(config["MY FAVORITE NUMBER"], ConfigValue::Float(3.14));
}

#[test]
fn test_float_whole_number() {
    let config = parse_ok("my favorite number = 42");
    assert_eq!(config["MY FAVORITE NUMBER"], ConfigValue::Float(42.0));
}

#[test]
fn test_float_negative() {
    let config = parse_ok("my favorite number = -2.718");
    assert_eq!(config["MY FAVORITE NUMBER"], ConfigValue::Float(-2.718));
}

#[test]
fn test_boolean_true_variants() {
    for val in &["true", "True", "TRUE", "yes", "Yes", "1", "on", "On"] {
        let input = format!("am i serious = {}", val);
        let config = parse_ok(&input);
        assert_eq!(
            config["AM I SERIOUS"],
            ConfigValue::Boolean(true),
            "Failed for boolean input: {}",
            val
        );
    }
}

#[test]
fn test_boolean_false_variants() {
    for val in &["false", "False", "FALSE", "no", "No", "0", "off", "Off"] {
        let input = format!("am i serious = {}", val);
        let config = parse_ok(&input);
        assert_eq!(
            config["AM I SERIOUS"],
            ConfigValue::Boolean(false),
            "Failed for boolean input: {}",
            val
        );
    }
}

#[test]
fn test_integer_collection() {
    let config = parse_ok("age of children = 1;2;3");
    assert_eq!(
        config["AGE OF CHILDREN"],
        ConfigValue::IntegerCollection(vec![1, 2, 3])
    );
}

#[test]
fn test_float_collection() {
    let config = parse_ok("size of children = 0.75;1.2;1.5");
    assert_eq!(
        config["SIZE OF CHILDREN"],
        ConfigValue::FloatCollection(vec![0.75, 1.2, 1.5])
    );
}

// #[test]
// fn test_path_bare() {
//     let config = parse_ok("where do i live = /my/house");
//     assert_eq!(
//         config["WHERE DO I LIVE"],
//         Value::Path(std::path::PathBuf::from("/my/house"))
//     );
// }

// #[test]
// fn test_path_quoted() {
//     let config = parse_ok(r#"where do i live = "/my/cozy house""#);
//     assert_eq!(
//         config["WHERE DO I LIVE"],
//         Value::Path(std::path::PathBuf::from("/my/cozy house"))
//     );
// }

// --- Case insensitivity ---

#[test]
fn test_key_case_insensitive_upper() {
    let config = parse_ok("WHO MOM LOVES = me");
    assert_eq!(config["WHO MOM LOVES"], ConfigValue::String("me".into()));
}

#[test]
fn test_key_mixed_case() {
    let config = parse_ok("Who Mom Loves = me");
    assert_eq!(config["WHO MOM LOVES"], ConfigValue::String("me".into()));
}

// --- Comments ---

#[test]
fn test_hash_comment_line() {
    let config = parse_ok("# this is a comment\nmy age = 30");
    assert_eq!(config["MY AGE"], ConfigValue::Integer(30));
}

#[test]
fn test_slash_comment_line() {
    let config = parse_ok("/ this is a comment\nmy age = 30");
    assert_eq!(config["MY AGE"], ConfigValue::Integer(30));
}

#[test]
fn test_inline_hash_comment() {
    let config = parse_ok("my age = 30 # my real age");
    assert_eq!(config["MY AGE"], ConfigValue::Integer(30));
}

#[test]
fn test_inline_slash_comment() {
    let config = parse_ok("my age = 30 / my real age");
    assert_eq!(config["MY AGE"], ConfigValue::Integer(30));
}

#[test]
fn test_comment_inside_quoted_string_preserved() {
    let config = parse_ok(r#"who mom loves = 'me / always'"#);
    assert_eq!(
        config["WHO MOM LOVES"],
        ConfigValue::String("me / always".into())
    );
}

#[test]
fn test_hash_inside_quoted_string_preserved() {
    let config = parse_ok(r#"who mom loves = 'me # always'"#);
    assert_eq!(
        config["WHO MOM LOVES"],
        ConfigValue::String("me # always".into())
    );
}

// --- Whitespace handling ---

#[test]
fn test_leading_trailing_whitespace_on_key() {
    let config = parse_ok("  who mom loves  = me");
    assert_eq!(config["WHO MOM LOVES"], ConfigValue::String("me".into()));
}

#[test]
fn test_leading_trailing_whitespace_on_value() {
    let config = parse_ok("my age =   25  ");
    assert_eq!(config["MY AGE"], ConfigValue::Integer(25));
}

#[test]
fn test_empty_lines_ignored() {
    let config = parse_ok("\n\nmy age = 25\n\n");
    assert_eq!(config["MY AGE"], ConfigValue::Integer(25));
}

// --- Multiple keys ---

#[test]
fn test_multiple_keys() {
    let input = indoc::indoc! {"
        who mom loves = me
        my age = 25
        am i serious = false
        / where do i live = \"/my/house\"
        my favorite number = 3.14
    "};
    let config = parse_ok(input);
    assert_eq!(config["WHO MOM LOVES"], ConfigValue::String("me".into()));
    assert_eq!(config["MY AGE"], ConfigValue::Integer(25));
    assert_eq!(config["AM I SERIOUS"], ConfigValue::Boolean(false));
    //assert_eq!(config["where do i live"], Value::Path(std::path::PathBuf::from("/my/house")));
    assert_eq!(config["MY FAVORITE NUMBER"], ConfigValue::Float(3.14));
}

#[test]
fn test_last_value_wins_on_duplicate_key() {
    let config = parse_ok("my age = 20\nmy age = 99");
    assert_eq!(config["MY AGE"], ConfigValue::Integer(99));
}

// --- Error cases ---

#[test]
fn test_unknown_key_produces_error() {
    let mut errors = parse_err("my cat = fluffy");
    let err0 = errors.pop().expect("should have one error reported");
    assert!(err0.is::<ParseError>());
    let parse_error: Box<ParseError> = err0.downcast().expect("not a ParseError");
    assert!(matches!(*parse_error, ParseError::UnknownKey { pos: _, key: k } if k == "my cat"));
}

#[test]
fn test_invalid_integer() {
    let mut errors = parse_err("my age = olderthandirt");
    let err0 = errors.pop().expect("should have one error reported");
    assert!(err0.is::<ParseError>());
    let parse_error: Box<ParseError> = err0.downcast().expect("not a ParseError");
    assert!(matches!(*parse_error, ParseError::InvalidValue { key, .. } if key == "my age"));
}

#[test]
fn test_out_of_bound_integer() {
    let mut errors = parse_err("my age = -5");
    let err0 = errors.pop().expect("should have one error reported");
    assert!(err0.is::<ParseError>());
    let parse_error: Box<ParseError> = err0.downcast().expect("not a ParseError");

    assert!(matches!(*parse_error, ParseError::OutOfBound { key, .. } if key == "my age"));
}

#[test]
fn test_out_of_bound_integer_collection() {
    let mut errors = parse_err("AGE OF CHILDREN = 1;-10;3;4");
    let err0 = errors.pop().expect("should have one error reported");
    assert!(err0.is::<ParseError>());
    let parse_error: Box<ParseError> = err0.downcast().expect("not a ParseError");

    assert!(matches!(*parse_error, ParseError::OutOfBound { key, .. } if key == "AGE OF CHILDREN"));
}

#[test]
fn test_out_of_bound_float_collection() {
    let mut errors = parse_err("SIZE OF CHILDREN = 0.75;0.01;1.5");
    let err0 = errors.pop().expect("should have one error reported");
    assert!(err0.is::<ParseError>());
    let parse_error: Box<ParseError> = err0.downcast().expect("not a ParseError");

    assert!(
        matches!(*parse_error, ParseError::OutOfBound { key, .. } if key == "SIZE OF CHILDREN")
    );
}

#[test]
fn test_invalid_float() {
    let mut errors = parse_err("my favorite number = 'a lot'");
    let err0 = errors.pop().expect("should have one error reported");
    assert!(err0.is::<ParseError>());
    let parse_error: Box<ParseError> = err0.downcast().expect("not a ParseError");

    assert!(
        matches!(*parse_error, ParseError::InvalidValue { key, .. } if key == "my favorite number")
    );
}

#[test]
fn test_invalid_boolean() {
    let mut errors = parse_err("am i serious = maybe");
    let err0 = errors.pop().expect("should have one error reported");
    assert!(err0.is::<ParseError>());
    let parse_error: Box<ParseError> = err0.downcast().expect("not a ParseError");

    assert!(matches!(*parse_error, ParseError::InvalidValue { key, .. } if key == "am i serious"));
}

#[test]
fn test_missing_equals_sign() {
    let mut errors = parse_err("my age 25");
    let err0 = errors.pop().expect("should have one error reported");
    assert!(err0.is::<DamoclesError>());
    let parse_error: Box<DamoclesError> = err0.downcast().expect("not a DamoclesError");

    assert!(matches!(
        *parse_error,
        DamoclesError::MissingEndValue { .. }
    ));
}

#[test]
fn test_multiple_errors_collected() {
    let input = indoc::indoc! {"
        my cat = fluffy
        my age = olderthandirt
        missing equals
    "};
    let errors = parse_err(input);
    assert_eq!(errors.len(), 3);
}

#[test]
fn test_valid_and_invalid_lines_mixed() {
    let input = "my age = 25\nmy cat = fluffy";
    let dico = make_dico();
    let parser = Parser::new(&dico);
    let errors = parser.parse(input).expect_err("should have errors");
    assert_eq!(errors.len(), 1);
}

#[test]
fn test_good_choice() {
    let input = "sexe des enfants = GARCON;girl";
    let config = parse_ok(input);

    // Choices are normalized
    assert_eq!(
        config["SEX OF CHILDREN"],
        ConfigValue::StringCollection(vec!["BOY".into(), "GIRL".into()])
    );
}

#[test]
fn test_wrong_choice() {
    let input = "sexe des enfants = dog;girl";
    let mut errors = parse_err(input);

    let err0 = errors.pop().expect("should have one error reported");
    assert!(err0.is::<ParseError>());
    let parse_error: Box<ParseError> = err0.downcast().expect("not a ParseError");

    assert!(matches!(*parse_error, ParseError::BadChoice { key, .. } if key == "sexe des enfants"));
}

// Ignore some french works for spelling
// cSpell:ignore NOM INDEX MNEMO NIVEAU CHOIX
// cSpell:ignore QUI MAMAN AIME
// cSpell:ignore MON AGE
// cSpell:ignore SEXE DES ENFANTS
// cSpell:ignore SUIS JE SERIEUX
// cSpell:ignore MON NOMBRE FAVORIS
