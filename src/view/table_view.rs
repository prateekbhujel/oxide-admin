use crate::resource::{FormMode, QueryState, Resource};
use crate::table::{Column, ColumnType, TableStyle};

pub fn render_table_partial(resource: &dyn Resource, query: &QueryState, user: &crate::domain::User) -> String {
    let table_def = resource.table();
    let (rows, total_count) = resource.fetch_rows(query);

    let per_page = if query.per_page == 0 { 8 } else { query.per_page };
    let current_page = if query.page == 0 { 1 } else { query.page };
    let total_pages = (total_count + per_page - 1) / per_page;
    let start_idx = if total_count == 0 { 0 } else { (current_page - 1) * per_page + 1 };
    let end_idx = (start_idx + rows.len()).saturating_sub(1);

    let cell_padding = match table_def.style {
        TableStyle::Compact => "px-3.5 py-2",
        _ => "px-5 py-3.5",
    };

    // Build Table Header
    let mut header_th = String::new();
    for col in &table_def.columns {
        let is_sorted = query.sort_by.as_deref() == Some(&col.name);
        let sort_icon = if is_sorted {
            if query.sort_desc {
                r#"<svg class="w-3.5 h-3.5 text-zinc-300 inline ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path></svg>"#
            } else {
                r#"<svg class="w-3.5 h-3.5 text-zinc-300 inline ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"></path></svg>"#
            }
        } else {
            ""
        };

        let click_attr = if col.sortable {
            format!(
                r#"onclick="onSortClick('{slug}', '{name}', '{cur_sort}', '{cur_desc}')" class="px-5 py-3 text-left text-[11px] font-medium text-zinc-400 tracking-wider uppercase cursor-pointer hover:text-zinc-200 select-none transition-colors""#,
                slug = resource.slug(),
                name = col.name,
                cur_sort = query.sort_by.as_deref().unwrap_or(""),
                cur_desc = query.sort_desc
            )
        } else {
            r#"class="px-5 py-3 text-left text-[11px] font-medium text-zinc-500 tracking-wider uppercase""#.to_string()
        };

        header_th.push_str(&format!(
            r#"<th scope="col" {click_attr}>
                <div class="flex items-center gap-1">
                    <span>{label}</span>
                    {sort_icon}
                </div>
            </th>"#,
            label = col.label,
            sort_icon = sort_icon,
            click_attr = click_attr
        ));
    }

    // Actions Header
    header_th.push_str(r#"<th scope="col" class="px-5 py-3 text-right text-[11px] font-medium text-zinc-500 tracking-wider uppercase">Actions</th>"#);

    // Build Table Body Rows
    let mut body_rows = String::new();
    if rows.is_empty() {
        let cols_len = table_def.columns.len() + 1;
        body_rows.push_str(&format!(
            r#"<tr><td colspan="{cols_len}" class="px-5 py-12 text-center text-zinc-500 text-xs font-mono">
                <div class="flex flex-col items-center justify-center gap-2">
                    <svg class="w-6 h-6 text-zinc-600" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"></path></svg>
                    <span>No records matching criteria.</span>
                </div>
            </td></tr>"#,
            cols_len = cols_len
        ));
    } else {
        let can_edit = resource.can_edit(user);
        let can_delete = resource.can_delete(user);
        let mode = resource.form_mode();

        for (idx, row) in rows.iter().enumerate() {
            let mut row_tds = String::new();

            for col in &table_def.columns {
                let raw_val = row.get(&col.name).unwrap_or("—");
                let rendered_cell = render_cell(col, raw_val);
                row_tds.push_str(&format!(r#"<td class="{cell_padding} whitespace-nowrap text-xs text-zinc-200">{rendered_cell}</td>"#));
            }

            // Edit and Delete Actions wired to mode with policy enforcement
            let row_values_json = serde_json::to_string(&row.values).unwrap_or_else(|_| "{}".to_string());
            let escaped_row_values_json = row_values_json.replace('"', "&quot;");

            let mut actions_html = String::new();
            if can_edit {
                let edit_btn = match mode {
                    FormMode::Page => format!(
                        r#"<a href="/admin/{slug}/edit/{id}" class="p-1 rounded text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 transition-colors inline-block" title="Edit">
                            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/><path d="m15 5 4 4"/></svg>
                        </a>"#,
                        slug = resource.slug(),
                        id = row.id
                    ),
                    _ => format!(
                        r#"<button type="button" onclick="openEditDialog('{slug}', '{id}', '{values}')" class="p-1 rounded text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 transition-colors" title="Edit">
                            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/><path d="m15 5 4 4"/></svg>
                        </button>"#,
                        slug = resource.slug(),
                        id = row.id,
                        values = escaped_row_values_json
                    ),
                };
                actions_html.push_str(&edit_btn);
            }
            if can_delete {
                actions_html.push_str(&format!(
                    r#"<button type="button" onclick="openDeleteDialog('{slug}', '{id}')" class="p-1 rounded text-zinc-500 hover:text-rose-400 hover:bg-rose-500/10 transition-colors" title="Delete">
                        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                    </button>"#,
                    slug = resource.slug(),
                    id = row.id
                ));
            }
            if !can_edit && !can_delete {
                actions_html = r#"<span class="text-zinc-600 text-xs select-none">—</span>"#.to_string();
            }

            row_tds.push_str(&format!(r#"<td class="{cell_padding} whitespace-nowrap text-xs text-right space-x-1">{actions_html}</td>"#));

            let row_bg_class = match table_def.style {
                TableStyle::Striped => {
                    if idx % 2 == 1 {
                        "bg-zinc-900/40 hover:bg-zinc-850/60"
                    } else {
                        "hover:bg-zinc-850/50"
                    }
                }
                _ => "hover:bg-zinc-850/50",
            };

            body_rows.push_str(&format!(r#"<tr class="{row_bg_class} transition-colors border-b border-zinc-800/80 last:border-b-0">{row_tds}</tr>"#));
        }
    }

    // Build Pagination HTML
    let prev_disabled = if current_page <= 1 { "opacity-30 pointer-events-none" } else { "" };
    let next_disabled = if current_page >= total_pages || total_pages == 0 { "opacity-30 pointer-events-none" } else { "" };

    let create_btn_html = if resource.can_create(user) {
        match resource.form_mode() {
            FormMode::Page => format!(
                r#"<a href="/admin/{slug}/create" class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-100 hover:bg-white text-zinc-900 font-medium text-xs transition-colors shadow-sm">
                    <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5v14"/></svg>
                    <span>New {name}</span>
                </a>"#,
                slug = resource.slug(),
                name = resource.name()
            ),
            _ => format!(
                r#"<button 
                    type="button" 
                    onclick="openCreateDialog()"
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-zinc-100 hover:bg-white text-zinc-900 font-medium text-xs transition-colors shadow-sm">
                    <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5v14"/></svg>
                    <span>New {name}</span>
                </button>"#,
                name = resource.name()
            ),
        }
    } else {
        String::new()
    };

    // Build Table Filters HTML
    let mut filters_html = String::new();
    for filter in &table_def.filters {
        let current_val = query.filters.get(&filter.name).map(|s| s.as_str()).unwrap_or("");
        let mut opts = format!(r#"<option value="">All {}</option>"#, filter.label);
        for (opt_val, opt_label) in &filter.options {
            let sel = if opt_val == current_val { "selected" } else { "" };
            opts.push_str(&format!(r#"<option value="{opt_val}" {sel}>{opt_label}</option>"#));
        }
        filters_html.push_str(&format!(
            r#"<select onchange="onFilterChange('{slug}', '{name}', this.value)" class="px-2.5 py-1.5 rounded-lg bg-zinc-900 border border-zinc-800 text-xs text-zinc-300 focus:outline-none focus:border-zinc-600 transition-colors font-sans">
                {opts}
            </select>"#,
            slug = resource.slug(),
            name = filter.name,
            opts = opts
        ));
    }

    format!(
        r#"<!-- Table Controls: Search, Filters & Create -->
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between pb-4 gap-3">
            <div class="flex items-center gap-2.5 flex-1 max-w-md">
                <div class="relative w-full">
                    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-zinc-500">
                        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
                    </div>
                    <input 
                        type="text" 
                        id="table-search-input"
                        value="{search_val}"
                        placeholder="Search {plural}..."
                        oninput="onSearchInput('{slug}', event)"
                        class="block w-full pl-8 pr-3 py-1.5 border border-zinc-800 rounded-lg bg-zinc-900 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none focus:ring-1 focus:ring-zinc-600 focus:border-zinc-600 transition-all font-sans"
                    />
                </div>
                {filters_html}
            </div>
            
            <div class="flex items-center gap-3 self-end sm:self-auto">
                <span class="text-[11px] text-zinc-500 font-mono">{total_count} records</span>
                {create_btn_html}
            </div>
        </div>

        <!-- Table Card -->
        <div class="bg-zinc-900 border border-zinc-800 rounded-xl overflow-hidden shadow-sm">
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-zinc-800">
                    <thead class="bg-zinc-850/60">
                        <tr>{header_th}</tr>
                    </thead>
                    <tbody class="divide-y divide-zinc-800/80">
                        {body_rows}
                    </tbody>
                </table>
            </div>

            <!-- Pagination Bar -->
            <div class="px-5 py-3 bg-zinc-900 border-t border-zinc-800 flex items-center justify-between text-[11px] text-zinc-500 font-mono">
                <div>
                    Showing <span class="font-medium text-zinc-300">{start_idx}–{end_idx}</span> of <span class="font-medium text-zinc-300">{total_count}</span>
                </div>
                <div class="flex items-center gap-1.5 font-sans">
                    <button 
                        type="button" 
                        onclick="onPageClick('{slug}', {prev_page})" 
                        class="px-2.5 py-1 rounded-md border border-zinc-800 bg-zinc-850 hover:bg-zinc-800 hover:text-zinc-200 transition-colors text-xs {prev_disabled}">
                        Prev
                    </button>
                    <span class="px-2 text-zinc-400 font-mono text-[11px]">{current_page}/{max_pages}</span>
                    <button 
                        type="button" 
                        onclick="onPageClick('{slug}', {next_page})" 
                        class="px-2.5 py-1 rounded-md border border-zinc-800 bg-zinc-850 hover:bg-zinc-800 hover:text-zinc-200 transition-colors text-xs {next_disabled}">
                        Next
                    </button>
                </div>
            </div>
        </div>"#,
        slug = resource.slug(),
        plural = resource.plural_name(),
        search_val = query.search,
        total_count = total_count,
        create_btn_html = create_btn_html,
        filters_html = filters_html,
        header_th = header_th,
        body_rows = body_rows,
        start_idx = start_idx,
        end_idx = end_idx,
        prev_page = current_page.saturating_sub(1),
        next_page = current_page + 1,
        max_pages = if total_pages == 0 { 1 } else { total_pages },
        prev_disabled = prev_disabled,
        next_disabled = next_disabled
    )
}

fn render_cell(col: &Column, val: &str) -> String {
    match &col.column_type {
        ColumnType::Text | ColumnType::Numeric => {
            if col.name == "id" || col.name == "email" || col.name == "amount" || col.name == "record_id" || col.name == "recordId" {
                format!(r#"<span class="font-mono text-zinc-300">{val}</span>"#)
            } else {
                format!(r#"<span class="font-medium text-zinc-100">{val}</span>"#)
            }
        }
        ColumnType::DateTime => {
            format!(r#"<span class="font-mono text-zinc-400">{val}</span>"#)
        }
        ColumnType::Badge { color_map } => {
            let color_classes = color_map
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(val))
                .map(|(_, c)| c.as_str())
                .unwrap_or("slate");

            let (badge_bg, badge_text, badge_border) = match color_classes {
                "emerald" | "green" => ("bg-emerald-500/10", "text-emerald-400", "border-emerald-500/20"),
                "indigo" | "blue" => ("bg-blue-500/10", "text-blue-400", "border-blue-500/20"),
                "amber" | "yellow" => ("bg-amber-500/10", "text-amber-400", "border-amber-500/20"),
                "rose" | "red" => ("bg-rose-500/10", "text-rose-400", "border-rose-500/20"),
                _ => ("bg-zinc-800", "text-zinc-400", "border-zinc-700/80"),
            };

            format!(
                r#"<span class="inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium border {badge_bg} {badge_text} {badge_border}">{val}</span>"#,
                badge_bg = badge_bg,
                badge_text = badge_text,
                badge_border = badge_border,
                val = val
            )
        }
    }
}
