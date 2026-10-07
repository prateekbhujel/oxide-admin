use crate::resource::{QueryState, Resource};
use crate::table::{ActionStyle, Column, ColumnType};

pub fn render_table_partial(resource: &dyn Resource, query: &QueryState) -> String {
    let table_def = resource.table();
    let (rows, total_count) = resource.fetch_rows(query);

    let per_page = if query.per_page == 0 { 10 } else { query.per_page };
    let current_page = if query.page == 0 { 1 } else { query.page };
    let total_pages = (total_count + per_page - 1) / per_page;
    let start_idx = if total_count == 0 { 0 } else { (current_page - 1) * per_page + 1 };
    let end_idx = (start_idx + rows.len()).saturating_sub(1);

    // Build Table Header
    let mut header_th = String::new();
    for col in &table_def.columns {
        let is_sorted = query.sort_by.as_deref() == Some(&col.name);
        let sort_icon = if is_sorted {
            if query.sort_desc {
                r#"<svg class="w-3.5 h-3.5 text-indigo-400 inline ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path></svg>"#
            } else {
                r#"<svg class="w-3.5 h-3.5 text-indigo-400 inline ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"></path></svg>"#
            }
        } else {
            ""
        };

        let click_attr = if col.sortable {
            format!(
                r#"onclick="onSortClick('{slug}', '{name}', '{cur_sort}', '{cur_desc}')" class="px-6 py-3.5 text-left text-xs font-semibold text-slate-300 uppercase tracking-wider cursor-pointer hover:text-white select-none transition-colors""#,
                slug = resource.slug(),
                name = col.name,
                cur_sort = query.sort_by.as_deref().unwrap_or(""),
                cur_desc = query.sort_desc
            )
        } else {
            r#"class="px-6 py-3.5 text-left text-xs font-semibold text-slate-400 uppercase tracking-wider""#.to_string()
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
    if !table_def.actions.is_empty() {
        header_th.push_str(r#"<th scope="col" class="px-6 py-3.5 text-right text-xs font-semibold text-slate-400 uppercase tracking-wider">Actions</th>"#);
    }

    // Build Table Body Rows
    let mut body_rows = String::new();
    if rows.is_empty() {
        let cols_len = table_def.columns.len() + if table_def.actions.is_empty() { 0 } else { 1 };
        body_rows.push_str(&format!(
            r#"<tr><td colspan="{cols_len}" class="px-6 py-12 text-center text-slate-500 text-sm">
                <div class="flex flex-col items-center justify-center gap-2">
                    <svg class="w-8 h-8 text-slate-600" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"></path></svg>
                    <span>No records found matching your criteria.</span>
                </div>
            </td></tr>"#,
            cols_len = cols_len
        ));
    } else {
        for row in &rows {
            let mut row_tds = String::new();

            for col in &table_def.columns {
                let raw_val = row.values.get(&col.name).map(|s| s.as_str()).unwrap_or("—");
                let rendered_cell = render_cell(col, raw_val);
                row_tds.push_str(&format!(r#"<td class="px-6 py-4 whitespace-nowrap text-sm text-slate-200">{rendered_cell}</td>"#));
            }

            // Actions Cell
            if !table_def.actions.is_empty() {
                let mut actions_html = String::new();
                for act in &table_def.actions {
                    let btn_class = match act.style {
                        ActionStyle::Danger => "text-rose-400 hover:text-rose-300 hover:bg-rose-500/10",
                        ActionStyle::Primary => "text-indigo-400 hover:text-indigo-300 hover:bg-indigo-500/10",
                        ActionStyle::Default => "text-slate-400 hover:text-slate-200 hover:bg-slate-800",
                    };
                    actions_html.push_str(&format!(
                        r#"<button type="button" class="px-2.5 py-1 text-xs rounded-md font-medium transition-colors {btn_class}">{label}</button>"#,
                        btn_class = btn_class,
                        label = act.label
                    ));
                }
                row_tds.push_str(&format!(r#"<td class="px-6 py-4 whitespace-nowrap text-sm text-right space-x-1">{actions_html}</td>"#));
            }

            body_rows.push_str(&format!(r#"<tr class="hover:bg-slate-800/40 transition-colors border-b border-slate-800/60 last:border-b-0">{row_tds}</tr>"#));
        }
    }

    // Build Pagination HTML
    let prev_disabled = if current_page <= 1 { "opacity-40 pointer-events-none" } else { "" };
    let next_disabled = if current_page >= total_pages || total_pages == 0 { "opacity-40 pointer-events-none" } else { "" };

    format!(
        r#"<!-- Table Controls: Live Search -->
        <div class="flex items-center justify-between pb-4 gap-4">
            <div class="relative max-w-sm w-full">
                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-slate-400">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
                </div>
                <input 
                    type="text" 
                    id="table-search-input"
                    value="{search_val}"
                    placeholder="Search {plural}..."
                    oninput="onSearchInput('{slug}', event)"
                    class="block w-full pl-9 pr-4 py-2 border border-slate-800 rounded-lg bg-slate-900/80 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500 transition-all"
                />
            </div>
            <div class="text-xs text-slate-400 flex items-center gap-2">
                <span>{total_count} total records</span>
            </div>
        </div>

        <!-- Table Card -->
        <div class="bg-slate-900/60 border border-slate-800/80 rounded-xl overflow-hidden shadow-xl backdrop-blur-sm">
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-slate-800/80">
                    <thead class="bg-slate-800/50">
                        <tr>{header_th}</tr>
                    </thead>
                    <tbody class="divide-y divide-slate-800/60">
                        {body_rows}
                    </tbody>
                </table>
            </div>

            <!-- Pagination Bar -->
            <div class="px-6 py-4 bg-slate-900/40 border-t border-slate-800/80 flex items-center justify-between text-xs text-slate-400">
                <div>
                    Showing <span class="font-semibold text-slate-200">{start_idx}</span> to <span class="font-semibold text-slate-200">{end_idx}</span> of <span class="font-semibold text-slate-200">{total_count}</span>
                </div>
                <div class="flex items-center gap-2">
                    <button 
                        type="button" 
                        onclick="onPageClick('{slug}', {prev_page})" 
                        class="px-3 py-1.5 rounded-lg border border-slate-800 bg-slate-800/60 hover:bg-slate-700 hover:text-white transition-colors {prev_disabled}">
                        Previous
                    </button>
                    <span class="px-2 text-slate-500 font-mono">Page {current_page} of {max_pages}</span>
                    <button 
                        type="button" 
                        onclick="onPageClick('{slug}', {next_page})" 
                        class="px-3 py-1.5 rounded-lg border border-slate-800 bg-slate-800/60 hover:bg-slate-700 hover:text-white transition-colors {next_disabled}">
                        Next
                    </button>
                </div>
            </div>
        </div>"#,
        slug = resource.slug(),
        plural = resource.plural_name(),
        search_val = query.search,
        total_count = total_count,
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
        ColumnType::Text | ColumnType::DateTime | ColumnType::Numeric => val.to_string(),
        ColumnType::Badge { color_map } => {
            let color_classes = color_map
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(val))
                .map(|(_, c)| c.as_str())
                .unwrap_or("slate");

            let (badge_bg, badge_text, badge_border) = match color_classes {
                "green" | "emerald" => ("bg-emerald-500/10", "text-emerald-400", "border-emerald-500/20"),
                "blue" | "indigo" => ("bg-indigo-500/10", "text-indigo-400", "border-indigo-500/20"),
                "yellow" | "amber" => ("bg-amber-500/10", "text-amber-400", "border-amber-500/20"),
                "red" | "rose" => ("bg-rose-500/10", "text-rose-400", "border-rose-500/20"),
                _ => ("bg-slate-800", "text-slate-400", "border-slate-700"),
            };

            format!(
                r#"<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border {badge_bg} {badge_text} {badge_border} capitalize">{val}</span>"#,
                badge_bg = badge_bg,
                badge_text = badge_text,
                badge_border = badge_border,
                val = val
            )
        }
    }
}
