/// # Integration tests for free_surface::config::casfile
mod test_helpers;

use free_surface::config::casfile;
use free_surface::config::configvalue::ConfigValue;
use free_surface::config::dicofile;

use test_helpers::telemac_file;

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn gouttedo_casfile() {
    let dico_path =
        telemac_file("sources/telemac2d/telemac2d.dico").expect("Can't get telemac file");
    let dico = dicofile::parse_file(dico_path).expect("dico is ok");

    let steering_path = telemac_file("examples/telemac2d/gouttedo/t2d_gouttedo.cas")
        .expect("Can't get steering file");

    let parser = casfile::Parser::new(&dico);

    let steering = parser
        .parse_file(steering_path)
        .expect("cas file parser failed");

    assert_eq!(steering.len(), 27);

    let check_value = |name: &str, expected_value: ConfigValue| {
        let value = steering.get(name).expect("no such variable");
        assert_eq!(*value, expected_value);
    };

    check_value(
        "EQUATIONS",
        ConfigValue::String(String::from("SAINT-VENANT FE")),
    );
    check_value(
        "FORTRAN FILE",
        ConfigValue::String(String::from("user_fortran")),
    );
    check_value(
        "BOUNDARY CONDITIONS FILE",
        ConfigValue::String(String::from("geo_gouttedo.cli")),
    );
    check_value(
        "GEOMETRY FILE",
        ConfigValue::String(String::from("geo_gouttedo.slf")),
    );
    check_value(
        "RESULTS FILE",
        ConfigValue::String(String::from("r2d_gouttedo_v1p0.slf")),
    );

    // OPTIONS GENERALES
    check_value(
        "TITLE",
        ConfigValue::String(String::from("TELEMAC 2D: DROPLET IN A BASIN")),
    );
    check_value(
        "VARIABLES FOR GRAPHIC PRINTOUTS",
        ConfigValue::String(String::from("U,V,H,T*")),
    );
    check_value("TIME STEP", ConfigValue::Float(0.04));

    check_value("NUMBER OF TIME STEPS", ConfigValue::Integer(100));
    check_value("GRAPHIC PRINTOUT PERIOD", ConfigValue::Integer(5));
    check_value("LISTING FOR PRINTOUT PERIOD", ConfigValue::Integer(10));
    check_value("LAW OF BOTTOM FRICTION", ConfigValue::Integer(3));
    check_value("FRICTION COEFFICIENT", ConfigValue::Float(40.));

    // PROPAGATION

    check_value("TURBULENCE MODEL", ConfigValue::Integer(1));
    check_value("VELOCITY DIFFUSIVITY", ConfigValue::Float(0.0));
    check_value("SOLVER", ConfigValue::Integer(7));
    check_value("SOLVER OPTION", ConfigValue::Integer(3));
    check_value(
        "MAXIMUM NUMBER OF ITERATIONS FOR SOLVER",
        ConfigValue::Integer(100),
    );
    check_value("SOLVER ACCURACY", ConfigValue::Float(1e-4));
    check_value("IMPLICITATION FOR DEPTH", ConfigValue::Float(0.6));
    check_value("IMPLICITATION FOR VELOCITY", ConfigValue::Float(0.6));

    // ------

    check_value("MASS-BALANCE", ConfigValue::Boolean(true));
    check_value(
        "INITIAL CONDITIONS",
        ConfigValue::String(String::from("PARTICULAR")),
    );
    check_value(
        "TYPE OF ADVECTION",
        ConfigValue::IntegerCollection([2, 5].to_vec()),
    );
    check_value(
        "SUPG OPTION",
        ConfigValue::IntegerCollection([2, 2].to_vec()),
    );
    check_value(
        "DISCRETIZATIONS IN SPACE",
        ConfigValue::IntegerCollection([11, 11].to_vec()),
    );

    check_value("TREATMENT OF THE LINEAR SYSTEM", ConfigValue::Integer(1));
}

// CSpell:ignore SUPG DISCRETIZATIONS
