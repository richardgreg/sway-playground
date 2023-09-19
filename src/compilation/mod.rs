use crate::{util::read_file_contents, types::CompileResponse};

use self::{
    swaypad::{clean_error_content, create_project, remove_project, write_main_file},
    tooling::{build_project, check_forc_version, switch_fuel_toolchain},
};
use hex::encode;
use std::fs::read_to_string;

pub mod swaypad;
pub mod tooling;

/// Build and destroy a project.
pub fn build_and_destroy_project(
    contract: String,
    toolchain: String,
) -> CompileResponse {
    // Check if any contract has been submitted.
    if contract.is_empty() {
        return CompileResponse {
            abi: "".to_string(),
            bytecode: "".to_string(),
            storage_slots: "".to_string(),
            error: "No contract.".to_string(),
            forc_version: "".to_string(),
        };
    }

    // Switch to the given fuel toolchain and check forc version.
    switch_fuel_toolchain(toolchain);
    let forc_version = check_forc_version();

    // Create a new project.
    let project_name = create_project().unwrap();

    // Write the file to the temporary project and compile.
    write_main_file(project_name.to_owned(), contract.as_bytes()).unwrap();
    let output = build_project(project_name.clone());

    // If the project compiled successfully, read the ABI and BIN files.
    if output.status.success() {
        let abi = read_to_string(format!(
            "projects/{}/out/debug/swaypad-abi.json",
            project_name
        ))
        .expect("Should have been able to read the file");
        let bin = read_file_contents(format!("projects/{}/out/debug/swaypad.bin", project_name));
        let storage_slots = read_file_contents(format!("projects/{}/out/debug/swaypad-storage_slots.json", project_name));

        // Remove the project directory and contents.
        remove_project(project_name).unwrap();

        // Return the abi, bin, empty error message, and forc version.
        CompileResponse {
            abi: clean_error_content(abi),
            bytecode: clean_error_content(encode(bin)),
            storage_slots: String::from_utf8_lossy(&storage_slots).into(),
            error: String::from(""),
            forc_version,
        }
    } else {
        // Get the error message presented in the console output.
        let error = std::str::from_utf8(&output.stderr).unwrap();

        // Get the index of the main file.
        let main_index = error.find("/main.sw:").unwrap();

        // Truncate the error message to only include the relevant content.
        let trunc = String::from(error).split_off(main_index);

        // Remove the project.
        remove_project(project_name).unwrap();

        // Return an empty abi, bin, error message, and forc version.
        CompileResponse {
            abi: String::from(""),
            bytecode: String::from(""),
            storage_slots: String::from(""),
            error: clean_error_content(trunc),
            forc_version,
        }
    }
}
