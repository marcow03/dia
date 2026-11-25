use wasm_bindgen::prelude::*;

/// Returns `{"diagram": Diagram, "warnings": [string]}` as JSON.
#[wasm_bindgen]
pub fn read(input: &str, from: &str) -> Result<String, JsError> {
    let mut warnings = vec![];
    let diagram = dia_core::read(input, from, &mut warnings)?;
    Ok(serde_json::json!({ "diagram": diagram, "warnings": warnings }).to_string())
}

#[wasm_bindgen]
pub fn write(diagram: &str, to: &str) -> Result<String, JsError> {
    let diagram: dia_core::Diagram = serde_json::from_str(diagram)?;
    Ok(dia_core::write(&diagram, to)?)
}

/// Returns `[{"name": string, "extensions": [string]}]` as JSON.
#[wasm_bindgen]
pub fn formats() -> String {
    let list: Vec<_> = dia_core::FORMATS
        .iter()
        .map(|f| serde_json::json!({ "name": f.name, "extensions": f.extensions }))
        .collect();
    serde_json::Value::from(list).to_string()
}
