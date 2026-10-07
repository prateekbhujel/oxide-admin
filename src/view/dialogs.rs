use crate::form::{FieldType, Form};
use crate::resource::Resource;

pub fn render_dialogs(resource: &dyn Resource) -> String {
    let form = resource.form();
    let slug = resource.slug();
    let name = resource.name();

    let create_fields_html = render_form_fields(&form, "create");
    let edit_fields_html = render_form_fields(&form, "edit");

    format!(
        r#"<!-- Native In-App Create Dialog -->
<dialog id="create-dialog" class="bg-zinc-900 border border-zinc-800 rounded-xl p-0 text-zinc-100 max-w-md w-full shadow-2xl backdrop:bg-black/75 backdrop:backdrop-blur-sm m-auto">
    <div class="flex items-center justify-between px-5 py-4 border-b border-zinc-800">
        <h3 class="text-sm font-medium text-zinc-100">New {name}</h3>
        <button type="button" onclick="document.getElementById('create-dialog').close()" class="text-zinc-500 hover:text-zinc-300 text-lg leading-none">&times;</button>
    </div>
    <form method="POST" action="/admin/{slug}/create" class="p-5 space-y-4">
        <div class="space-y-3">
            {create_fields_html}
        </div>
        <div class="pt-3 border-t border-zinc-800 flex items-center justify-end gap-2">
            <button type="button" onclick="document.getElementById('create-dialog').close()" class="px-3 py-1.5 rounded-lg border border-zinc-800 text-xs text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors">
                Cancel
            </button>
            <button type="submit" class="px-4 py-1.5 rounded-lg bg-zinc-100 hover:bg-white text-zinc-900 font-medium text-xs transition-colors">
                Create {name}
            </button>
        </div>
    </form>
</dialog>

<!-- Native In-App Edit Dialog -->
<dialog id="edit-dialog" class="bg-zinc-900 border border-zinc-800 rounded-xl p-0 text-zinc-100 max-w-md w-full shadow-2xl backdrop:bg-black/75 backdrop:backdrop-blur-sm m-auto">
    <div class="flex items-center justify-between px-5 py-4 border-b border-zinc-800">
        <h3 id="edit-dialog-title" class="text-sm font-medium text-zinc-100">Edit {name}</h3>
        <button type="button" onclick="document.getElementById('edit-dialog').close()" class="text-zinc-500 hover:text-zinc-300 text-lg leading-none">&times;</button>
    </div>
    <form id="edit-form" method="POST" action="" class="p-5 space-y-4">
        <div class="space-y-3">
            {edit_fields_html}
        </div>
        <div class="pt-3 border-t border-zinc-800 flex items-center justify-end gap-2">
            <button type="button" onclick="document.getElementById('edit-dialog').close()" class="px-3 py-1.5 rounded-lg border border-zinc-800 text-xs text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors">
                Cancel
            </button>
            <button type="submit" class="px-4 py-1.5 rounded-lg bg-zinc-100 hover:bg-white text-zinc-900 font-medium text-xs transition-colors">
                Save Changes
            </button>
        </div>
    </form>
</dialog>

<!-- Native In-App Delete Confirmation Dialog (Replaces browser alert) -->
<dialog id="delete-dialog" class="bg-zinc-900 border border-zinc-800 rounded-xl p-0 text-zinc-100 max-w-sm w-full shadow-2xl backdrop:bg-black/75 backdrop:backdrop-blur-sm m-auto">
    <div class="p-5">
        <div class="flex items-start gap-3.5 mb-4">
            <div class="w-8 h-8 rounded-lg bg-rose-500/10 border border-rose-500/20 flex items-center justify-center shrink-0 text-rose-400">
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/></svg>
            </div>
            <div>
                <h4 class="text-sm font-medium text-zinc-100">Delete record</h4>
                <p class="text-xs text-zinc-400 mt-1">Are you sure you want to delete this record? This action cannot be undone.</p>
            </div>
        </div>
        <form id="delete-form" method="POST" action="" class="flex items-center justify-end gap-2 pt-2 border-t border-zinc-800">
            <button type="button" onclick="document.getElementById('delete-dialog').close()" class="px-3 py-1.5 rounded-lg border border-zinc-800 text-xs text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors">
                Cancel
            </button>
            <button type="submit" class="px-4 py-1.5 rounded-lg bg-rose-600 hover:bg-rose-500 text-white font-medium text-xs transition-colors shadow-sm">
                Delete
            </button>
        </form>
    </div>
</dialog>"#,
        name = name,
        slug = slug,
        create_fields_html = create_fields_html,
        edit_fields_html = edit_fields_html
    )
}

fn render_form_fields(form: &Form, prefix: &str) -> String {
    let mut out = String::new();

    for f in &form.fields {
        let input_id = format!("{prefix}-field-{}", f.name);
        let placeholder = f.placeholder.as_deref().unwrap_or("");
        let required = if f.required { "required" } else { "" };

        let field_input = match &f.field_type {
            FieldType::Select { options } => {
                let mut opts_html = String::new();
                for (val, label) in options {
                    opts_html.push_str(&format!(
                        r#"<option value="{val}">{label}</option>"#
                    ));
                }
                format!(
                    r#"<select id="{input_id}" name="{name}" {required} class="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200 focus:outline-none focus:border-zinc-600 transition-colors">{opts_html}</select>"#,
                    input_id = input_id,
                    name = f.name,
                    required = required,
                    opts_html = opts_html
                )
            }
            FieldType::Email => {
                format!(
                    r#"<input type="email" id="{input_id}" name="{name}" placeholder="{placeholder}" {required} class="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-zinc-600 transition-colors" />"#,
                    input_id = input_id,
                    name = f.name,
                    placeholder = placeholder,
                    required = required
                )
            }
            _ => {
                format!(
                    r#"<input type="text" id="{input_id}" name="{name}" placeholder="{placeholder}" {required} class="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-zinc-600 transition-colors" />"#,
                    input_id = input_id,
                    name = f.name,
                    placeholder = placeholder,
                    required = required
                )
            }
        };

        out.push_str(&format!(
            r#"<div>
                <label for="{input_id}" class="block text-xs font-medium text-zinc-300 mb-1.5">{label}</label>
                {field_input}
            </div>"#,
            input_id = input_id,
            label = f.label,
            field_input = field_input
        ));
    }

    out
}
