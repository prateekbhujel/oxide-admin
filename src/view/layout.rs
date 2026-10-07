use crate::resource::DynResource;

pub fn render_page(
    title: &str,
    current_slug: &str,
    resources: &[DynResource],
    content_html: &str,
    flash_message: Option<&str>,
) -> String {
    let mut sidebar_nav = String::new();

    for res in resources {
        let is_active = res.slug() == current_slug;
        let active_classes = if is_active {
            "bg-zinc-800/80 text-zinc-100 font-medium border-l-2 border-zinc-100"
        } else {
            "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-850/60"
        };

        let icon_svg = match res.slug() {
            "users" => r#"<svg class="w-4 h-4 shrink-0 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>"#,
            "orders" => r#"<svg class="w-4 h-4 shrink-0 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><circle cx="8" cy="21" r="1"/><circle cx="19" cy="21" r="1"/><path d="M2.05 2.05h2l2.66 12.42a2 2 0 0 0 2 1.58h9.78a2 2 0 0 0 1.95-1.57l1.65-7.43H5.12"/></svg>"#,
            _ => r#"<svg class="w-4 h-4 shrink-0 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/></svg>"#,
        };

        sidebar_nav.push_str(&format!(
            r#"<a href="/admin/{slug}" class="flex items-center gap-2.5 px-3 py-2 rounded-md text-xs transition-colors {active_classes}">
                {icon_svg}
                <span>{name}</span>
            </a>"#,
            slug = res.slug(),
            name = res.plural_name(),
            icon_svg = icon_svg,
            active_classes = active_classes
        ));
    }

    let toast_html = if let Some(msg) = flash_message {
        format!(
            r#"<div id="flash-toast" class="fixed bottom-5 right-5 z-50 flex items-center gap-2.5 px-4 py-3 rounded-lg bg-zinc-900 border border-zinc-700/80 shadow-2xl text-xs text-zinc-200 animate-fade-in">
                <svg class="w-4 h-4 text-emerald-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path></svg>
                <span>{msg}</span>
                <button type="button" onclick="document.getElementById('flash-toast').remove()" class="ml-2 text-zinc-500 hover:text-zinc-300">&times;</button>
            </div>"#
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} · OxideAdmin</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/geist@1.3.0/dist/fonts/geist-sans/style.css">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/geist@1.3.0/dist/fonts/geist-mono/style.css">
    <script>
        tailwind.config = {{
            darkMode: 'class',
            theme: {{
                extend: {{
                    fontFamily: {{
                        sans: ['"Geist Sans"', 'system-ui', '-apple-system', 'sans-serif'],
                        mono: ['"Geist Mono"', 'ui-monospace', 'monospace'],
                    }},
                    colors: {{
                        zinc: {{
                            950: '#09090b',
                            900: '#121215',
                            850: '#18181c',
                            800: '#27272a',
                            750: '#323238',
                            700: '#3f3f46',
                            500: '#71717a',
                            400: '#a1a1aa',
                            300: '#d4d4d8',
                            100: '#fafafa',
                        }}
                    }}
                }}
            }}
        }}
    </script>
    <style>
        body {{ font-family: 'Geist Sans', system-ui, -apple-system, sans-serif; }}
        .loading-shimmer {{ opacity: 0.6; pointer-events: none; transition: opacity 0.15s ease; }}
        @keyframes fadeIn {{ from {{ opacity: 0; transform: translateY(6px); }} to {{ opacity: 1; transform: translateY(0); }} }}
        .animate-fade-in {{ animation: fadeIn 0.2s cubic-bezier(0.16, 1, 0.3, 1) forwards; }}
    </style>
</head>
<body class="bg-zinc-950 text-zinc-100 min-h-screen flex antialiased selection:bg-zinc-800 selection:text-white">
    <!-- Sidebar -->
    <aside class="w-60 bg-zinc-900 border-r border-zinc-800 flex flex-col shrink-0">
        <!-- Brand Header -->
        <div class="h-14 flex items-center justify-between px-4 border-b border-zinc-800">
            <div class="flex items-center gap-2.5">
                <div class="w-7 h-7 rounded-lg bg-zinc-800 border border-zinc-700/80 flex items-center justify-center">
                    <svg class="w-4 h-4 text-zinc-100" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polygon points="12 2 2 7 12 12 22 7 12 2"></polygon>
                        <polyline points="2 17 12 22 22 17"></polyline>
                        <polyline points="2 12 12 17 22 12"></polyline>
                    </svg>
                </div>
                <div>
                    <span class="text-xs font-semibold tracking-tight text-zinc-100">Oxide<span class="text-zinc-400 font-normal">Admin</span></span>
                </div>
            </div>
            <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400 border border-zinc-700/60">Rust</span>
        </div>

        <!-- Navigation Links -->
        <div class="p-3 flex-1 space-y-1">
            <div class="px-2 pb-1.5 text-[10px] font-semibold tracking-wider text-zinc-500 uppercase font-mono">WORKSPACE</div>
            <nav class="space-y-0.5">
                {sidebar_nav}
            </nav>
        </div>

        <!-- User Profile & Sign Out -->
        <div class="p-3 border-t border-zinc-800 flex items-center justify-between">
            <div class="flex items-center gap-2 min-w-0">
                <div class="w-6 h-6 rounded-full bg-zinc-800 border border-zinc-700 flex items-center justify-center text-[10px] font-mono font-medium text-zinc-300">
                    PB
                </div>
                <div class="min-w-0">
                    <span class="block text-xs font-medium text-zinc-200 truncate">Pratik Bhujel</span>
                    <span class="block text-[10px] text-zinc-500 font-mono truncate">Lead Architect</span>
                </div>
            </div>
            <a href="/admin/logout" title="Sign Out" class="p-1 rounded text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800 transition-colors">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.75" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"></path></svg>
            </a>
        </div>
    </aside>

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col min-w-0 overflow-y-auto">
        <!-- Topbar -->
        <header class="h-14 border-b border-zinc-800 bg-zinc-950/80 flex items-center justify-between px-8 backdrop-blur-md sticky top-0 z-10">
            <div class="flex items-center gap-2 text-xs">
                <span class="text-zinc-500">Resources</span>
                <span class="text-zinc-600">/</span>
                <span class="text-zinc-200 font-medium">{title}</span>
            </div>

            <div class="flex items-center gap-3">
                <span class="text-[11px] font-mono text-zinc-400 bg-zinc-900 border border-zinc-800 px-2 py-0.5 rounded">
                    Engine: Axum 0.7
                </span>
            </div>
        </header>

        <!-- Page Body -->
        <div class="p-8 max-w-7xl w-full mx-auto">
            {content_html}
        </div>
    </main>

    {toast_html}

    <!-- Universal Modal Container for Create/Edit -->
    <div id="modal-backdrop" class="fixed inset-0 bg-black/70 backdrop-blur-sm z-40 hidden flex items-center justify-center p-4">
        <div id="modal-box" class="bg-zinc-900 border border-zinc-800 rounded-xl max-w-md w-full shadow-2xl overflow-hidden animate-fade-in">
            <div class="flex items-center justify-between px-5 py-4 border-b border-zinc-800">
                <h3 id="modal-title" class="text-sm font-medium text-zinc-100">Create Record</h3>
                <button type="button" onclick="closeModal()" class="text-zinc-500 hover:text-zinc-300 text-lg leading-none">&times;</button>
            </div>
            <form id="modal-form" method="POST" action="" class="p-5 space-y-4">
                <div id="modal-fields" class="space-y-3">
                    <!-- Dynamic form fields injected here -->
                </div>
                <div class="pt-3 border-t border-zinc-800 flex items-center justify-end gap-2">
                    <button type="button" onclick="closeModal()" class="px-3 py-1.5 rounded-lg border border-zinc-800 text-xs text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors">
                        Cancel
                    </button>
                    <button type="submit" id="modal-submit-btn" class="px-4 py-1.5 rounded-lg bg-zinc-100 hover:bg-white text-zinc-900 font-medium text-xs transition-colors">
                        Save Record
                    </button>
                </div>
            </form>
        </div>
    </div>

    <!-- Client Script for Live Updates & Modals -->
    <script>
        let searchTimeout;
        function updateTable(slug, params = {{}}) {{
            const container = document.getElementById('table-wrapper');
            if (container) container.classList.add('loading-shimmer');

            const url = new URL(window.location.origin + '/admin/' + slug + '/table');
            const searchInput = document.getElementById('table-search-input');
            if (searchInput && searchInput.value) {{
                url.searchParams.set('search', searchInput.value);
            }}
            for (const [k, v] of Object.entries(params)) {{
                url.searchParams.set(k, v);
            }}

            fetch(url.toString(), {{ headers: {{ 'X-Requested-With': 'XMLHttpRequest' }} }})
                .then(r => r.text())
                .then(html => {{
                    if (container) {{
                        container.innerHTML = html;
                        container.classList.remove('loading-shimmer');
                        const newSearch = document.getElementById('table-search-input');
                        if (newSearch && params.search_cursor) {{
                            newSearch.focus();
                            newSearch.setSelectionRange(params.search_cursor, params.search_cursor);
                        }}
                    }}
                }})
                .catch(err => {{
                    console.error('Update failed', err);
                    if (container) container.classList.remove('loading-shimmer');
                }});
        }}

        function onSearchInput(slug, e) {{
            clearTimeout(searchTimeout);
            const cursor = e.target.selectionStart;
            searchTimeout = setTimeout(() => {{
                updateTable(slug, {{ search_cursor: cursor }});
            }}, 150);
        }}

        function onSortClick(slug, column, currentSort, currentDesc) {{
            const isSame = currentSort === column;
            const newDesc = isSame ? (currentDesc === 'true' ? 'false' : 'true') : 'false';
            updateTable(slug, {{ sort_by: column, sort_desc: newDesc }});
        }}

        function onPageClick(slug, page) {{
            updateTable(slug, {{ page: page }});
        }}

        // Modal Controls
        function openCreateModal(slug, resourceName, fieldsJson) {{
            const fields = JSON.parse(fieldsJson);
            document.getElementById('modal-title').textContent = 'New ' + resourceName;
            document.getElementById('modal-form').action = '/admin/' + slug + '/create';
            document.getElementById('modal-submit-btn').textContent = 'Create ' + resourceName;

            let fieldsHtml = '';
            for (const f of fields) {{
                fieldsHtml += `
                    <div>
                        <label class="block text-xs font-medium text-zinc-300 mb-1">$&#123;f.label&#125;</label>
                        $&#123;renderFieldInput(f, '')&#125;
                    </div>
                `;
            }}
            document.getElementById('modal-fields').innerHTML = fieldsHtml;
            document.getElementById('modal-backdrop').classList.remove('hidden');
        }}

        function openEditModal(slug, resourceName, id, fieldsJson, valuesJson) {{
            const fields = JSON.parse(fieldsJson);
            const values = JSON.parse(valuesJson);
            document.getElementById('modal-title').textContent = 'Edit ' + resourceName + ' #' + id;
            document.getElementById('modal-form').action = '/admin/' + slug + '/edit/' + id;
            document.getElementById('modal-submit-btn').textContent = 'Update ' + resourceName;

            let fieldsHtml = '';
            for (const f of fields) {{
                const val = values[f.name] || '';
                fieldsHtml += `
                    <div>
                        <label class="block text-xs font-medium text-zinc-300 mb-1">$&#123;f.label&#125;</label>
                        $&#123;renderFieldInput(f, val)&#125;
                    </div>
                `;
            }}
            document.getElementById('modal-fields').innerHTML = fieldsHtml;
            document.getElementById('modal-backdrop').classList.remove('hidden');
        }}

        function renderFieldInput(f, val) {{
            if (f.field_type.Select) {{
                let opts = '';
                for (const [k, label] of f.field_type.Select.options) {{
                    const sel = (k === val) ? 'selected' : '';
                    opts += `<option value="$&#123;k&#125;" $&#123;sel&#125;>$&#123;label&#125;</option>`;
                }}
                return `<select name="$&#123;f.name&#125;" class="w-full px-3 py-1.5 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200 focus:outline-none focus:border-zinc-600">$&#123;opts&#125;</select>`;
            }}
            const type = f.field_type === 'Email' ? 'email' : 'text';
            return `<input type="$&#123;type&#125;" name="$&#123;f.name&#125;" value="$&#123;val&#125;" required class="w-full px-3 py-1.5 bg-zinc-950 border border-zinc-800 rounded-lg text-xs text-zinc-200 focus:outline-none focus:border-zinc-600" />`;
        }}

        function closeModal() {{
            document.getElementById('modal-backdrop').classList.add('hidden');
        }}

        function confirmDelete(slug, id) {{
            if (confirm('Are you sure you want to delete this record?')) {{
                const form = document.createElement('form');
                form.method = 'POST';
                form.action = '/admin/' + slug + '/delete/' + id;
                document.body.appendChild(form);
                form.submit();
            }}
        }}

        // Auto dismiss flash toast
        setTimeout(() => {{
            const t = document.getElementById('flash-toast');
            if (t) t.remove();
        }}, 4000);
    </script>
</body>
</html>"#,
        title = title,
        sidebar_nav = sidebar_nav,
        content_html = content_html,
        toast_html = toast_html
    )
}
