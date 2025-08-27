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

        // Always-available core verbs
        v.push(ActionDoc {
            name: "ucm_core",
            prompt_snippet: r#"
Use **Unified Command Markdown (UCM)** blocks:

```get {#f1}
file:Cargo.toml
dir:src/
glob:src/**/*.rs
mem:long
````

```set { target="file:README.md" }
New note
```

```run { sh=true }
git status -sb
```

"#.trim().to_string(),
            when: |_| true,
        });

        Self { actions: v }
    }

    pub fn select_for(&self, ctx: &ActionContext) -> Vec<&ActionDoc> {
        self.actions.iter().filter(|a| (a.when)(ctx)).collect()
    }
}