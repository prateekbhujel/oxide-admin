use std::collections::HashMap;
use crate::form::{FieldType, Form};
use crate::resource::Resource;

pub fn render_form_page(
    resource: &dyn Resource,
    id: Option<&str>,
    values: Option<&HashMap<String, String>>,
) -> String {
    let form = resource.form();
    let slug = resource.slug();
    let name = resource.name();
    let is_edit = id.is_some();

    let (title, form_action) = if let Some(rec_id) = id {
        (format!("Edit {name} #{rec_id}"), format!("/admin/{slug}/edit/{rec_id}"))
    } else {
        (format!("Create {name}"), format!("/admin/{slug}/create"))
    };

    let fields_html = render_page_fields(&form, values);

    format!(
        r#"<div class="max-w-2xl mx-auto space-y-6">
            <!-- Header with Back Link -->
            <div class="flex items-center justify-between">
                <div>
                    <a href="/admin/{slug}" class="text-xs text-zinc-500 hover:text-zinc-300 flex items-center gap-1.5 transition-colors mb-2">
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path></svg>
                        <span>Back to {plural}</span>
                    </a>
                    <h1 class="text-xl font-semibold tracking-tight text-zinc-100">{title}</h1>
                </div>
            </div>

            <!-- Form Card -->
            <div class="bg-zinc-900 border border-zinc-800 rounded-xl p-6 shadow-sm">
                <form method="POST" action="{form_action}" class="space-y-5">
                    <div class="space-y-4">
                        {fields_html}
                    </div>

                    <div class="pt-5 border-t border-zinc-800 flex items-center justify-end gap-3">
                        <a href="/admin/{slug}" class="px-3.5 py-2 rounded-lg border border-zinc-800 text-xs font-medium text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors">
                            Cancel
                        </a>
                        <button type="submit" class="px-5 py-2 rounded-lg bg-zinc-100 hover:bg-white text-zinc-900 font-medium text-xs transition-colors shadow-sm">
                            {btn_text}
                        </button>
                    </div>
                </form>
            </div>
        </div>"#,
        slug = slug,
        plural = resource.plural_name(),
        title = title,
        form_action = form_action,
        fields_html = fields_html,
        btn_text = if is_edit { "Save Changes" } else { "Create Record" }
    )
}

fn render_page_fields(form: &Form, values: Option<&HashMap<String, String>>) -> String {
    let mut out = String::new();

    for f in &form.fields {
        let input_id = format!("page-field-{}", f.name);
        let placeholder = f.placeholder.as_deref().unwrap_or("");
        let required = if f.required { "required" } else { "" };
        let current_val = values
            .and_then(|v| {
                v.get(&f.name)
                    .or_else(|| v.get(&crate::resource::to_camel_case(&f.name)))
                    .or_else(|| v.get(&crate::resource::to_snake_case(&f.name)))
            })
            .map(|s| s.as_str())
            .unwrap_or("");

        let field_input = match &f.field_type {
            FieldType::Select { options } => {
                let mut opts_html = String::new();
                for (val, label) in options {
                    let sel = if val == current_val { "selected" } else { "" };
                    opts_html.push_str(&format!(
                        r#"<option value="{val}" {sel}>{label}</option>"#
                    ));
                }
                format!(
                    r#"<select id="{input_id}" name="{name}" {required} class="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200 focus:outline-none focus:border-zinc-600 transition-colors font-sans">{opts_html}</select>"#,
                    input_id = input_id,
                    name = f.name,
                    required = required,
                    opts_html = opts_html
                )
            }
            FieldType::SearchableSelect { options } => {
                let active_val = if !current_val.is_empty() {
                    current_val
                } else {
                    options.first().map(|(v, _)| v.as_str()).unwrap_or("")
                };
                let active_lbl = options.iter().find(|(v, _)| v == active_val).map(|(_, l)| l.as_str()).unwrap_or(active_val);

                let mut items_html = String::new();
                for (val, label) in options {
                    items_html.push_str(&format!(
                        r#"<div onclick="selectSearchableOption('{input_id}', '{val}', '{label}')" data-label="{label}" class="searchable-opt px-2.5 py-1.5 text-xs text-zinc-300 hover:bg-zinc-800 hover:text-white rounded cursor-pointer transition-colors flex items-center justify-between"><span>{label}</span><span class="text-[10px] text-zinc-500 font-mono">{val}</span></div>"#
                    ));
                }
                format!(
                    r#"<div class="relative" id="container-{input_id}">
                        <input type="hidden" id="{input_id}" name="{name}" value="{active_val}" {required} />
                        <button type="button" onclick="toggleSearchableSelect('{input_id}')" id="btn-{input_id}" class="w-full flex items-center justify-between px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200 focus:outline-none focus:border-zinc-600 transition-colors">
                            <span id="label-{input_id}" class="truncate">{active_lbl}</span>
                            <svg class="w-3.5 h-3.5 text-zinc-500 shrink-0 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path></svg>
                        </button>
                        <div id="dropdown-{input_id}" class="hidden absolute z-50 left-0 right-0 mt-1 bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl p-1.5 max-h-52 overflow-y-auto backdrop-blur-md">
                            <div class="p-1 border-b border-zinc-800 mb-1">
                                <input type="text" placeholder="Search options..." oninput="filterSearchableSelect('{input_id}', this.value)" class="w-full px-2 py-1 bg-zinc-950 border border-zinc-800 rounded text-xs text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-zinc-600 font-sans" />
                            </div>
                            <div class="space-y-0.5" id="opts-{input_id}">
                                {items_html}
                            </div>
                        </div>
                    </div>"#,
                    input_id = input_id,
                    name = f.name,
                    required = required,
                    active_val = active_val,
                    active_lbl = active_lbl,
                    items_html = items_html
                )
            }
            FieldType::Email => {
                format!(
                    r#"<input type="email" id="{input_id}" name="{name}" value="{current_val}" placeholder="{placeholder}" {required} class="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-zinc-600 transition-colors" />"#,
                    input_id = input_id,
                    name = f.name,
                    current_val = current_val,
                    placeholder = placeholder,
                    required = required
                )
            }
            _ => {
                format!(
                    r#"<input type="text" id="{input_id}" name="{name}" value="{current_val}" placeholder="{placeholder}" {required} class="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200 placeholder-zinc-600 focus:outline-none focus:border-zinc-600 transition-colors" />"#,
                    input_id = input_id,
                    name = f.name,
                    current_val = current_val,
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
