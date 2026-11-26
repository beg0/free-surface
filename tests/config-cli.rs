// # Integration tests for free_surface::config::cli

mod test_helpers;

use free_surface::config::cli::{BoundariesConditions, BoundaryCondition, BoundaryType};

use test_helpers::{fixture, telemac_file};

#[test]
#[ignore = "downloads telemac source, run with --include-ignored"]
fn telemac_cli() {
    let cli_files = [
        "examples/artemis/rides/geo_rides.cli",
        "examples/artemis/bosse_elliptique/geo_shoal.cli",
        "examples/telemac2d/estimation/geo_estimation.cli",
    ];

    for relative_path in cli_files {
        let full_path = telemac_file(relative_path).expect("Can't get telemac file");
        let (_bc, diag) = BoundariesConditions::parse_file_best_effort(full_path);
        if diag.has_errors(false) {
            panic!(
                "{}",
                diag.all()
                    .iter()
                    .map(|e| format!("{}: {:?}: {}\n", e.loc, e.severity, e.message))
                    .collect::<Vec<String>>()
                    .concat()
            );
        }
    }
}

#[test]
fn parse_from_file() {
    let filename = fixture("slf_boundaries.cli");
    let (bc, diag) = BoundariesConditions::parse_file_best_effort(filename);

    let expected_bc = vec![
        BoundaryCondition {
            h_boundary_type: BoundaryType::IncidentWave,
            u_boundary_type: BoundaryType::Input,
            v_boundary_type: BoundaryType::Input,
            h_prescription: 1.0,
            u_prescription: 1.1,
            v_prescription: 1.2,
            friction_coefficient: 1.3,
            tracer_boundary_type: BoundaryType::Wall,
            tracer_prescription: 1.4,
            at_prescription: 1.5,
            bt_prescription: 1.6,
            vertice_idx: 101,
            color: 1,
        },
        BoundaryCondition {
            h_boundary_type: BoundaryType::Output,
            u_boundary_type: BoundaryType::Input,
            v_boundary_type: BoundaryType::Input,
            h_prescription: 2.0,
            u_prescription: 2.1,
            v_prescription: 2.2,
            friction_coefficient: 2.3,
            tracer_boundary_type: BoundaryType::Input,
            tracer_prescription: 2.4,
            at_prescription: 2.5,
            bt_prescription: 2.6,
            vertice_idx: 102,
            color: 2,
        },
        BoundaryCondition {
            h_boundary_type: BoundaryType::Wall,
            u_boundary_type: BoundaryType::Wall,
            v_boundary_type: BoundaryType::Wall,
            h_prescription: 3.0,
            u_prescription: 3.1,
            v_prescription: 3.2,
            friction_coefficient: 3.3,
            tracer_boundary_type: BoundaryType::Wall,
            tracer_prescription: 3.4,
            at_prescription: 3.5,
            bt_prescription: 3.6,
            vertice_idx: 103,
            color: 3,
        },
        BoundaryCondition {
            h_boundary_type: BoundaryType::Wall,
            u_boundary_type: BoundaryType::Input,
            v_boundary_type: BoundaryType::Input,
            h_prescription: 4.0,
            u_prescription: 4.1,
            v_prescription: 4.2,
            friction_coefficient: 4.3,
            tracer_boundary_type: BoundaryType::Wall,
            tracer_prescription: 4.4,
            at_prescription: 4.5,
            bt_prescription: 4.6,
            vertice_idx: 104,
            color: 4,
        },
        BoundaryCondition {
            h_boundary_type: BoundaryType::Wall,
            u_boundary_type: BoundaryType::Input,
            v_boundary_type: BoundaryType::Input,
            h_prescription: 5.0,
            u_prescription: 5.1,
            v_prescription: 5.2,
            friction_coefficient: 5.3,
            tracer_boundary_type: BoundaryType::Wall,
            tracer_prescription: 5.4,
            at_prescription: 5.5,
            bt_prescription: 5.6,
            vertice_idx: 105,
            color: 5,
        },
        BoundaryCondition {
            h_boundary_type: BoundaryType::Wall,
            u_boundary_type: BoundaryType::InputVelocity,
            v_boundary_type: BoundaryType::InputVelocity,
            h_prescription: 6.0,
            u_prescription: 6.1,
            v_prescription: 6.2,
            friction_coefficient: 6.3,
            tracer_boundary_type: BoundaryType::Wall,
            tracer_prescription: 6.4,
            at_prescription: 6.5,
            bt_prescription: 6.6,
            vertice_idx: 106,
            color: 6,
        },
    ];
    // this file should
    assert!(!diag.has_errors(false));
    assert_eq!(diag.all().len(), 0);

    assert_eq!(bc.len(), 6);
    let as_vec: Vec<BoundaryCondition> = bc.iter().map(BoundaryCondition::clone).collect();
    assert_eq!(as_vec, expected_bc);
}

// ignore name in path of telemac sources
// cSpell:ignore bosse_elliptique
