pub fn render_login_page(error_msg: Option<&str>) -> String {
    let error_banner = if let Some(err) = error_msg {
        format!(
            r#"<div class="mb-5 p-3 rounded-md bg-rose-500/10 border border-rose-500/20 text-xs text-rose-400 flex items-center gap-2">
                <svg class="w-4 h-4 shrink-0 text-rose-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                <span>{err}</span>
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
    <title>Sign In · OxideAdmin</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/geist@1.3.0/dist/fonts/geist-sans/style.css">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/geist@1.3.0/dist/fonts/geist-mono/style.css">
    <script>
        tailwind.config = {{
            darkMode: 'class',
            theme: {{
                extend: {{
                    fontFamily: {{
                        sans: ['"Geist Sans"', 'system-ui', 'sans-serif'],
                        mono: ['"Geist Mono"', 'monospace'],
                    }},
                    colors: {{
                        zinc: {{
                            950: '#09090b',
                            900: '#121215',
                            850: '#1a1a1f',
                            800: '#27272a',
                            700: '#3f3f46',
                            400: '#a1a1aa',
                            300: '#d4d4d8',
                            100: '#f4f4f5',
                        }}
                    }}
                }}
            }}
        }}
    </script>
    <style>
        body {{ font-family: 'Geist Sans', system-ui, sans-serif; }}
    </style>
</head>
<body class="bg-zinc-950 text-zinc-100 min-h-screen flex items-center justify-center p-4 antialiased selection:bg-zinc-800 selection:text-white">
    <div class="w-full max-w-sm">
        <!-- Brand Header -->
        <div class="flex flex-col items-center mb-8">
            <div class="w-10 h-10 rounded-xl bg-zinc-900 border border-zinc-800 flex items-center justify-center mb-4 shadow-sm">
                <!-- Geometric Minimalist Monogram -->
                <svg class="w-5 h-5 text-zinc-100" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polygon points="12 2 2 7 12 12 22 7 12 2"></polygon>
                    <polyline points="2 17 12 22 22 17"></polyline>
                    <polyline points="2 12 12 17 22 12"></polyline>
                </svg>
            </div>
            <h1 class="text-xl font-medium tracking-tight text-zinc-100">Sign in to OxideAdmin</h1>
            <p class="text-xs text-zinc-500 mt-1">Sign in to your account to continue</p>
        </div>

        <!-- Auth Card -->
        <div class="bg-zinc-900/80 border border-zinc-800 rounded-xl p-6 shadow-2xl backdrop-blur-md">
            {error_banner}

            <form method="POST" action="/admin/login" class="space-y-4">
                <div>
                    <label class="block text-xs font-medium text-zinc-300 mb-1.5" for="email">Email address</label>
                    <input 
                        type="email" 
                        id="email" 
                        name="email" 
                        value="pratik.bhujel@oxideadmin.dev"
                        required 
                        class="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-sm text-zinc-100 placeholder-zinc-500 focus:outline-none focus:ring-1 focus:ring-zinc-600 focus:border-zinc-600 transition-all font-sans"
                        placeholder="admin@example.com"
                    />
                </div>

                <div>
                    <div class="flex items-center justify-between mb-1.5">
                        <label class="text-xs font-medium text-zinc-300" for="password">Password</label>
                        <span class="text-[11px] text-zinc-500 font-mono">admin123</span>
                    </div>
                    <input 
                        type="password" 
                        id="password" 
                        name="password" 
                        value="admin123"
                        required 
                        class="w-full px-3 py-2 bg-zinc-950 border border-zinc-800 rounded-lg text-sm text-zinc-100 placeholder-zinc-500 focus:outline-none focus:ring-1 focus:ring-zinc-600 focus:border-zinc-600 transition-all font-sans"
                    />
                </div>

                <div class="pt-2">
                    <button 
                        type="submit" 
                        class="w-full py-2 px-4 rounded-lg bg-zinc-100 hover:bg-white text-zinc-900 font-medium text-sm transition-all duration-150 shadow-sm flex items-center justify-center gap-2">
                        <span>Continue</span>
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3"></path></svg>
                    </button>
                </div>
            </form>

            <!-- Quick Demo Role Switcher -->
            <div class="mt-6 pt-5 border-t border-zinc-800">
                <div class="text-[11px] font-medium text-zinc-400 mb-2">Switch Demo Account:</div>
                <div class="grid grid-cols-2 gap-2 text-left">
                    <button type="button" onclick="setDemo('pratik.bhujel@oxideadmin.dev')" class="p-2 rounded-lg bg-zinc-950/60 hover:bg-zinc-850 border border-zinc-800 text-left transition-colors">
                        <div class="text-xs font-medium text-zinc-200">Pratik Bhujel</div>
                        <div class="text-[10px] text-indigo-400 font-mono">Superadmin</div>
                    </button>
                    <button type="button" onclick="setDemo('dharma.shrestha@oxideadmin.dev')" class="p-2 rounded-lg bg-zinc-950/60 hover:bg-zinc-850 border border-zinc-800 text-left transition-colors">
                        <div class="text-xs font-medium text-zinc-200">Dharma Raj</div>
                        <div class="text-[10px] text-blue-400 font-mono">Admin</div>
                    </button>
                    <button type="button" onclick="setDemo('lasta.chaudhary@oxideadmin.dev')" class="p-2 rounded-lg bg-zinc-950/60 hover:bg-zinc-850 border border-zinc-800 text-left transition-colors">
                        <div class="text-xs font-medium text-zinc-200">Lasta Chaudhary</div>
                        <div class="text-[10px] text-amber-400 font-mono">Editor</div>
                    </button>
                    <button type="button" onclick="setDemo('ranjan.gumanju@oxideadmin.dev')" class="p-2 rounded-lg bg-zinc-950/60 hover:bg-zinc-850 border border-zinc-800 text-left transition-colors">
                        <div class="text-xs font-medium text-zinc-200">Ranjan Gumanju</div>
                        <div class="text-[10px] text-zinc-400 font-mono">Member (Read-only)</div>
                    </button>
                </div>
            </div>
        </div>
    </div>

    <script>
        function setDemo(email) {{
            document.getElementById('email').value = email;
            document.getElementById('password').value = 'admin123';
        }}
    </script>
</body>
</html>"#
    )
}
