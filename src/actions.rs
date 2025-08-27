use std::collections::HashSet;

pub struct ActionContext<'a> {
    pub user_input: &'a str,
    pub cwd: &'a str,
    pub has_interpreters: HashSet<&'static str>, // e.g. {"python3","node","bash","pwsh"}
}

pub struct ActionDoc {
    /// What we show in the prompt (tool syntax + when to use)
    pub prompt_snippet: String,
    /// Cheap predicate: should this be shown for current turn?
    pub when: fn(&ActionContext) -> bool,
    /// Stable name for metrics
    pub name: &'static str,
}

pub struct ActionRegistry {
    actions: Vec<ActionDoc>,
}

impl ActionRegistry {
    pub fn default() -> Self {
        let mut v = Vec::new();

        // Always-available core tools (shell/file/memory)
        v.push(ActionDoc {
            name: "core_prime_tools",
            prompt_snippet: r#"```primeactions
shell: <command>
list_dir: <path>
read_file: <path> [lines=START-END]
write_file: <path> [append=true]
<content>
EOF_PRIME
```"#.trim().to_string(),
            when: |_| true,
        });

        // RunScript tool (only if we have at least one interpreter)
        v.push(ActionDoc {
            name: "run_script",
            prompt_snippet: r#"Use when writing a short script is simpler than inline shell:
```primeactions
run_script: lang=<python|node|bash|pwsh|ruby|php> [args="..."] [timeout=30]
<code>
EOF_PRIME
```"#.trim().to_string(),
            when: |ctx| !ctx.has_interpreters.is_empty(),
        });

        // Memory tools (always available, but document briefly)
        v.push(ActionDoc {
            name: "memory_tools",
            prompt_snippet: r#"Memory:
```primeactions
write_memory: long_term
<content>
EOF_PRIME
clear_memory: short_term
```"#.trim().to_string(),
            when: |_| true,
        });

        Self { actions: v }
    }

    pub fn select_for(&self, ctx: &ActionContext) -> Vec<&ActionDoc> {
        self.actions.iter().filter(|a| (a.when)(ctx)).collect()
    }
}