//!
//! This build script automatically generates test tasks for various
//! versions of MongoDB.
//!

use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};

static FILE_NAME: &str = "suite-tasks.yml";

// Add new versions to this list
static VERSIONS: &[&str] = &["6.0", "7.0", "8.0", "rapid", "latest"];

// The three topologies available in driver tools are server, replica_set, and sharded_cluster.
// We only use server here, but tag it as standalone to make it clear that it's not a replica set or sharded cluster.
// TOPOLOGY: (tag name used in evergreen, topology name used by orchestration)
static TOPOLOGIES: &[(&str, &str)] = &[("standalone", "server")];

// If more test types are added, add them here.
// TEST_TYPE: (test name seed in evergreen, description in evergreen, cargo test flags)
static TEST_TYPES: &[(&str, &str, &str, &str)] = &[
    (
        "spec-query",
        "../testdata/spec_tests/query_tests",
        "run spec query tests",
        "--features=query --package=e2e-tests",
    ),
    (
        "index-usage",
        "../testdata/index_usage",
        "run index usage tests",
        "--features=index --package=e2e-tests",
    ),
    (
        "rust-e2e",
        "../testdata/e2e_tests",
        "run rust e2e tests",
        "--features=e2e --package=e2e-tests",
    ),
    (
        "rust-errors",
        "../testdata/errors",
        "run rust error tests",
        "--features=error --package=e2e-tests",
    ),
    (
        "schema-derivation",
        "../testdata/schema_derivation_tests",
        "run schema derivation tests",
        "--features=schema_derivation --package=e2e-tests",
    ),
];

fn main() {
    let test_template = r#"
  - name: test-{{test_name}}_{{version}}-{{topology_name}}
    tags: [{{version}}, {{topology_name}}]
    commands:
      - func: "bootstrap mongo-orchestration"
        vars:
          MONGODB_VERSION: {{version}}
          TOPOLOGY: {{topology}}
      - func: "install rust toolchain"
      - func: "run data loader"
        vars:
          working_dir: ${common_test_infra_dir}
          data_loader_args: "--mongod-uri mongodb://localhost:27017 -d {{test_data_path}}"
      - func: "run rust tests"
        vars:
          description: {{description}}
          cargo_test_flags: {{flags}}
    "#;

    let mut file = write_evergreen_test_file(FILE_NAME);

    writeln!(
        file,
        "# Generated by create-tasks.  Do not manually edit!\n# See mongosql-rs/evergreen/create-tasks/build.rs\n\ntasks:"
    )
    .unwrap();

    for &version in VERSIONS.iter() {
        for &(topology_name, topology) in TOPOLOGIES.iter() {
            for &(test_name, test_data_path, description, flags) in TEST_TYPES.iter() {
                writeln!(
                    file,
                    "{}",
                    test_template
                        .replace("{{test_name}}", test_name)
                        .replace("{{test_data_path}}", test_data_path)
                        .replace("{{description}}", description)
                        .replace("{{flags}}", flags)
                        .replace("{{version}}", version)
                        .replace("{{topology_name}}", topology_name)
                        .replace("{{topology}}", topology)
                )
                .unwrap();
            }
        }
    }
}

fn write_evergreen_test_file(file_name: &str) -> File {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let file_path = Path::new(&manifest_dir).parent().unwrap().join(file_name);
    match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&file_path)
    {
        Ok(file) => file,
        Err(e) => panic!("{e}: {file_path:?}"),
    }
}
