use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, prelude::*};
use std::process::Command;

fn replace(path: &str, old: &str, new: &str) -> io::Result<()> {
    let contents = fs::read_to_string(path)?;
    let new = contents.replace(old, new);
    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    file.write(new.as_bytes())?;
    Ok(())
}

fn main() {
    let package_version = env!("CARGO_PKG_VERSION");
    let package_name = env!("CARGO_PKG_NAME");
    let current_dir = env!("CARGO_MANIFEST_DIR");
    eprintln!("Current working directory: {current_dir}");
    eprintln!("Downloading the api specification...");
    Command::new("wget")
        .arg("https://raw.githubusercontent.com/manticoresoftware/openapi/master/manticore.yml")
        .arg("-c")
        .arg("-P")
        .arg(&current_dir)
        .status()
        .unwrap();
    eprintln!("Generating files from the api specification...");
    // https://github.com/OpenAPITools/openapi-generator/blob/master/docs/generators/rust.md
    Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(&format!("{current_dir}:/local"))
        .arg("openapitools/openapi-generator-cli")
        .arg("generate")
        .arg("-i")
        .arg("/local/manticore.yml")
        .arg("-g")
        .arg("rust")
        .arg("-o")
        .arg(&format!("/local"))
        .arg("--additional-properties")
        .arg("library=reqwest")
        .arg("--additional-properties")
        .arg(&format!("packageName={package_name}"))
        .arg("--additional-properties")
        .arg(&format!("packageVersion={package_version}"))
        .arg("--additional-properties")
        .arg("preferUnsignedInt=true")
        .arg("--additional-properties")
        .arg("supportMiddleware=true")
        .status()
        .unwrap();

    // sudo chown -R ${USER}:${USER} src/apis/ src/models/
    replace(
        "src/models/search_request.rs",
        r#"rename = "source""#,
        r#"rename = "_source""#,
    )
    .unwrap();
    replace("src/models/match_filter.rs", r#"Serialize, "#, "").unwrap();
    replace(
        "src/apis/utils_api.rs",
        r#"local_var_req_builder = local_var_req_builder.json(&body);"#,
        r#"local_var_req_builder = if let Some(true) = raw_response {
        local_var_req_builder.form(&[("query", body), ("mode", "raw")])
    } else {
        local_var_req_builder.form(&[("query", body)])
    };"#,
    )
    .unwrap();

    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.openapi-generator-ignore");
    println!("cargo:rerun-if-changed={current_dir}/manticore.yml");
}
