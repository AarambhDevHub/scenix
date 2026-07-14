use alloc::string::String;

use scenix_core::{InspectorId, InspectorItem, InspectorSnapshot, InspectorValue};

/// Hover/click response from rendering one inspector snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EguiInspectorResponse {
    /// Last hovered inspector item.
    pub hovered: Option<InspectorId>,
    /// Last clicked inspector item.
    pub activated: Option<InspectorId>,
}

/// Renders a read-only inspector tree in an existing egui UI.
pub fn show_inspector(ui: &mut egui::Ui, snapshot: &InspectorSnapshot) -> EguiInspectorResponse {
    let mut output = EguiInspectorResponse::default();
    for item in &snapshot.roots {
        show_item(ui, item, &mut output);
    }
    output
}

fn show_item(ui: &mut egui::Ui, item: &InspectorItem, output: &mut EguiInspectorResponse) {
    let title = if item.kind.is_empty() {
        item.label.clone()
    } else {
        alloc::format!("{}  ·  {}", item.label, item.kind)
    };
    let response = egui::CollapsingHeader::new(title)
        .id_salt(item.id.0)
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new(("scenix-inspector-fields", item.id.0))
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for field in &item.fields {
                        ui.label(&field.name);
                        ui.monospace(format_value(&field.value));
                        ui.end_row();
                    }
                });
            for child in &item.children {
                show_item(ui, child, output);
            }
        })
        .header_response;
    if response.hovered() {
        output.hovered = Some(item.id);
    }
    if response.clicked() {
        output.activated = Some(item.id);
    }
}

fn format_value(value: &InspectorValue) -> String {
    match value {
        InspectorValue::Bool(value) => alloc::format!("{value}"),
        InspectorValue::Integer(value) => alloc::format!("{value}"),
        InspectorValue::Unsigned(value) => alloc::format!("{value}"),
        InspectorValue::Number(value) => alloc::format!("{value:.4}"),
        InspectorValue::Text(value) => value.clone(),
        InspectorValue::Vec2(value) => alloc::format!("[{:.3}, {:.3}]", value.x, value.y),
        InspectorValue::Vec3(value) => {
            alloc::format!("[{:.3}, {:.3}, {:.3}]", value.x, value.y, value.z)
        }
        InspectorValue::Vec4(value) => alloc::format!(
            "[{:.3}, {:.3}, {:.3}, {:.3}]",
            value.x,
            value.y,
            value.z,
            value.w
        ),
        InspectorValue::Color(value) => alloc::format!(
            "rgba({:.3}, {:.3}, {:.3}, {:.3})",
            value.r,
            value.g,
            value.b,
            value.a
        ),
        InspectorValue::Bytes(value) if *value >= 1_048_576 => {
            alloc::format!("{:.2} MiB", *value as f64 / 1_048_576.0)
        }
        InspectorValue::Bytes(value) if *value >= 1024 => {
            alloc::format!("{:.2} KiB", *value as f64 / 1024.0)
        }
        InspectorValue::Bytes(value) => alloc::format!("{value} B"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scenix_core::{InspectorField, InspectorItem};

    #[test]
    fn headless_context_renders_snapshot() {
        let context = egui::Context::default();
        let mut snapshot = InspectorSnapshot::new();
        let mut item = InspectorItem::new(InspectorId(7), "Node", "mesh");
        item.fields.push(InspectorField::new("visible", true));
        snapshot.push(item);
        let output = context.run(Default::default(), |context| {
            egui::CentralPanel::default().show(context, |ui| {
                let _ = show_inspector(ui, &snapshot);
            });
        });
        assert!(!output.shapes.is_empty());
    }
}
