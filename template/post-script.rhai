if variable::get("wasm") {
    file::rename("project-wasm.json5", "project.json5")
} else {
    file::rename("project-no-wasm.json5", "project.json5")
}
