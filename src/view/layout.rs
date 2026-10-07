use crate::resource::DynResource;

pub fn render_page(
    title: &str,
    current_slug: &str,
    resources: &[DynResource],
    content_html: &str,
) -> String {
    let mut sidebar_nav = String::new();

    for res in resources {
        let is_active = res.slug() == current_slug;
        let active_classes = if is_active {
            "bg-indigo-600/10 text-indigo-400 font-semibold border-r-2 border-indigo-500"
        } else {
            "text-slate-400 hover:text-slate-200 hover:bg-slate-800/50"
        };

        sidebar_nav.push_str(&format!(
            r#"<a href="/admin/{slug}" class="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-all duration-150 {active_classes}">
                <svg class="w-4 h-4 opacity-70" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path></svg>
                <span>{name}</span>
            </a>"#,
            slug = res.slug(),
            name = res.plural_name(),
            active_classes = active_classes
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} · OxideAdmin</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {{
            darkMode: 'class',
            theme: {{
                extend: {{
                    colors: {{
                        brand: {{
                            50: '#eef2ff',
                            500: '#6366f1',
                            600: '#4f46e5',
                            700: '#4338ca',
                        }}
                    }}
                }}
            }}
        }}
    </script>
    <style>
        @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');
        body {{ font-family: 'Inter', sans-serif; }}
        /* Micro pulse animation on live search */
        .loading-shimmer {{
            opacity: 0.6;
            pointer-events: none;
            transition: opacity 0.15s ease;
        }}
    </style>
</head>
<body class="bg-slate-950 text-slate-100 min-h-screen flex antialiased selection:bg-indigo-500 selection:text-white">
    <!-- Sidebar -->
    <aside class="w-64 bg-slate-900/60 border-r border-slate-800/80 flex flex-col backdrop-blur-xl shrink-0">
        <!-- Brand Header -->
        <div class="h-16 flex items-center gap-3 px-6 border-b border-slate-800/80">
            <div class="w-8 h-8 rounded-lg bg-gradient-to-tr from-indigo-600 to-violet-500 flex items-center justify-center shadow-lg shadow-indigo-500/20 font-bold text-white text-base">
                ⚡
            </div>
            <div>
                <span class="font-bold text-slate-100 text-sm tracking-wide">Oxide<span class="text-indigo-400">Admin</span></span>
                <span class="block text-[10px] text-slate-500 font-mono tracking-wider uppercase">High-Performance Rust</span>
            </div>
        </div>

        <!-- Navigation Links -->
        <div class="p-4 flex-1 space-y-1">
            <div class="px-3 pb-2 text-[11px] font-semibold tracking-wider text-slate-500 uppercase">Resources</div>
            <nav class="space-y-1">
                {sidebar_nav}
            </nav>
        </div>

        <!-- Footer / Watermark -->
        <div class="p-4 border-t border-slate-800/80 text-[11px] text-slate-500 flex items-center justify-between">
            <span>v0.1.0 (Axum 0.7)</span>
            <span class="px-2 py-0.5 rounded bg-slate-800 text-slate-400 font-mono">0.02ms</span>
        </div>
    </aside>

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col min-w-0 overflow-y-auto">
        <!-- Topbar -->
        <header class="h-16 border-b border-slate-800/80 bg-slate-900/30 flex items-center justify-between px-8 backdrop-blur-md sticky top-0 z-10">
            <h1 class="text-lg font-semibold text-slate-200">{title}</h1>
            <div class="flex items-center gap-4">
                <span class="text-xs text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 px-2.5 py-1 rounded-full font-medium flex items-center gap-1.5">
                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                    Single Binary Active
                </span>
                <div class="w-8 h-8 rounded-full bg-slate-800 border border-slate-700 flex items-center justify-center text-xs font-semibold text-slate-300">
                    PB
                </div>
            </div>
        </header>

        <!-- Page Body -->
        <div class="p-8 max-w-7xl w-full mx-auto">
            {content_html}
        </div>
    </main>

    <!-- Lightweight 1KB Client-Side Driver (Reactive Table Fetcher) -->
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
                        // re-focus search if updated
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
    </script>
</body>
</html>"#,
        title = title,
        sidebar_nav = sidebar_nav,
        content_html = content_html
    )
}
