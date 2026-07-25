use crate::i18n::I18n;
use anyhow::{Context, Result, anyhow};
use domain::Config;
use domain::{Task, TaskPriority, TaskStatus};
use jiff::Zoned;
use minijinja::{Environment, context};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};
use serde::Serialize;
use std::fmt::Write as _;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::sync::OnceLock;
use std::time::Duration;
use std::time::SystemTime;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use tracing::{info, warn};

/// Number of days to retain EML files (older files are deleted on startup)
const EML_RETENTION_DAYS: u64 = 5;
/// Theme name used for syntax highlighting in code blocks
const CODE_THEME_NAME: &str = "base16-ocean.dark";

// Serializable data structures passed to the templates
#[derive(Serialize)]
struct TemplateSummary {
    total: usize,
    completed: usize,
    in_progress: usize,
    pending: usize,
    canceled: usize,
}

#[derive(Serialize)]
struct TemplateTask {
    id: i32,
    title: String,
    project: String,
    priority_text: String,
    priority_color: &'static str,
    status_text: String,
    status_color: &'static str,
    start_date: String,
    due_date: String,
    time_spent: f64,
    progress: f64,
    detail_html: String, // Converted HTML goes here
}

/// Returns the display text and color code for a task status
fn status_display(status: TaskStatus) -> (String, &'static str) {
    match status {
        TaskStatus::Pending => (fl!("status-pending"), "#6c757d"),
        TaskStatus::WorkInProgress => (fl!("status-work-in-progress"), "#007bff"),
        TaskStatus::Complete => (fl!("status-complete"), "#28a745"),
        TaskStatus::Canceled => (fl!("status-canceled"), "#ff5733"),
    }
}

/// Returns the display text and color code for a task priority
fn priority_display(priority: TaskPriority) -> (String, &'static str) {
    match priority {
        TaskPriority::High => (fl!("priority-high"), "#dc3545"),
        TaskPriority::Medium => (fl!("priority-medium"), "#ffc107"),
        TaskPriority::Low => (fl!("priority-low"), "#20c997"),
    }
}

/// Normalizes all line endings to CRLF (\r\n)
fn normalize_to_crlf(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\n', "\r\n")
}

/// `SyntaxSet` is expensive to load, so it's initialized once per process and reused
fn syntax_set() -> &'static SyntaxSet {
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// Same caching strategy for `ThemeSet`
fn theme_set() -> &'static ThemeSet {
    static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();
    THEME_SET.get_or_init(ThemeSet::load_defaults)
}

/// Macro that collapses the boilerplate of registering each mail template in `mail_env()`.
/// `include_str!` expands relative to the macro's call site, so this works fine
/// as long as the macro is only used from within this file.
macro_rules! add_mail_template {
    ($env:expr, $name:literal, $path:literal) => {
        $env.add_template($name, include_str!($path))
            .with_context(|| format!("Failed to compile {} mail template", $name))?;
    };
}

/// Returns a cached `minijinja::Environment` for mail templates
fn mail_env() -> Result<&'static Environment<'static>> {
    static MAIL_ENV: OnceLock<Environment<'static>> = OnceLock::new();
    if let Some(env) = MAIL_ENV.get() {
        return Ok(env);
    }
    let mut env = Environment::new();

    add_mail_template!(env, "mail_de", "../assets/mail_de.html");
    add_mail_template!(env, "mail_en", "../assets/mail_en.html");
    add_mail_template!(env, "mail_es", "../assets/mail_es.html");
    add_mail_template!(env, "mail_ja", "../assets/mail_ja.html");
    add_mail_template!(env, "mail_vi", "../assets/mail_vi.html");
    add_mail_template!(env, "mail_zh", "../assets/mail_zh.html");

    Ok(MAIL_ENV.get_or_init(|| env))
}

/// Applies minimal escaping so text can be safely embedded as HTML
fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn md_to_html(markdown_input: &str) -> String {
    let ps = syntax_set();
    let ts = theme_set();
    let theme = &ts.themes[CODE_THEME_NAME];

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    options.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);
    options.insert(Options::ENABLE_SUPERSCRIPT);
    options.insert(Options::ENABLE_SUBSCRIPT);
    options.insert(Options::ENABLE_GFM);
    let parser = Parser::new_ext(markdown_input, options);

    let mut new_events = Vec::new();
    let mut in_code_block = false;
    let mut current_lang = String::new();
    let mut code_accumulator = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                current_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                    pulldown_cmark::CodeBlockKind::Indented => String::new(),
                };
                code_accumulator.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let syntax = ps
                    .find_syntax_by_token(&current_lang)
                    .unwrap_or_else(|| ps.find_syntax_plain_text());

                let highlighted_html =
                    highlighted_html_for_string(&code_accumulator, ps, syntax, theme)
                        .unwrap_or_else(|e| {
                            warn!("Failed to generate syntax highlighting: {e}");
                            format!("<pre><code>{}</code></pre>", escape_html(&code_accumulator))
                        });

                new_events.push(Event::Html(highlighted_html.into()));
            }
            Event::Text(text) if in_code_block => {
                code_accumulator.push_str(&text);
            }
            _ => {
                if !in_code_block {
                    new_events.push(event);
                }
            }
        }
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, new_events.into_iter());
    html_output
}

/// Aggregates per-status counts from the task list in a single pass
fn summarize(tasks: &[Task]) -> TemplateSummary {
    let mut summary = TemplateSummary {
        total: tasks.len(),
        completed: 0,
        in_progress: 0,
        pending: 0,
        canceled: 0,
    };

    for task in tasks {
        match task.status {
            TaskStatus::Complete => summary.completed += 1,
            TaskStatus::WorkInProgress => summary.in_progress += 1,
            TaskStatus::Pending => summary.pending += 1,
            TaskStatus::Canceled => summary.canceled += 1,
        }
    }

    summary
}

fn to_template_task(task: &Task) -> TemplateTask {
    let (status_text, status_color) = status_display(task.status);
    let (priority_text, priority_color) = priority_display(task.priority);

    let detail_html = if task.detail.trim().is_empty() {
        format!(
            "<span style=\"color: #a0aec0; font-style: italic;\">{}</span>",
            fl!("no-additional-details")
        )
    } else {
        md_to_html(&task.detail)
    };

    TemplateTask {
        id: task.id,
        title: task.title.clone(),
        project: task.project.clone(),
        priority_text,
        priority_color,
        status_text,
        status_color,
        start_date: task.start_date.strftime("%Y/%m/%d").to_string(),
        due_date: task.due_date.strftime("%Y/%m/%d").to_string(),
        time_spent: f64::from(task.time_spent),
        progress: f64::from(task.progress),
        detail_html,
    }
}

pub fn create_mail_html(tasks: &[Task], image_data: &str) -> Result<String> {
    let summary = summarize(tasks);
    let date_headline = Zoned::now().date().strftime("%B %d, %Y").to_string();
    let template_tasks: Vec<TemplateTask> = tasks.iter().map(to_template_task).collect();

    let locale = I18n::global().get_locale()?;

    let template_name = match locale.language.as_str() {
        "ja" => "mail_ja",
        "de" => "mail_de",
        "zh" => "mail_zh",
        "vi" => "mail_vi",
        "es" => "mail_es",
        _ => "mail_en", // default to english
    };

    let tmpl = mail_env()?.get_template(template_name)?;

    let rendered = tmpl.render(context! {
        image_data => image_data,
        date_headline => date_headline,
        summary => summary,
        tasks => template_tasks,
    })?;

    // Normalize all line endings to CRLF to satisfy the test requirements
    Ok(normalize_to_crlf(&rendered))
}

fn cleanup_old_eml_files(dir: &std::path::Path, retention_days: u64) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    let now = SystemTime::now();
    let max_age = Duration::from_secs(retention_days * 24 * 60 * 60);

    let is_stale_eml = |entry: &fs::DirEntry| -> bool {
        let path = entry.path();
        let is_eml = path.is_file() && path.extension().is_some_and(|ext| ext == "eml");
        if !is_eml {
            return false;
        }
        entry
            .metadata()
            .and_then(|meta| meta.modified())
            .map(|modified| now.duration_since(modified).unwrap_or_default())
            .is_ok_and(|age| age > max_age)
    };

    fs::read_dir(dir)?
        .flatten() // Keep only the Ok(entry) values
        .filter(is_stale_eml)
        .for_each(|entry| {
            let path = entry.path();
            match fs::remove_file(&path) {
                Ok(()) => info!("Deleted old eml file: {}", path.display()),
                Err(e) => warn!("Failed to delete old eml file {}: {}", path.display(), e),
            }
        });

    Ok(())
}

/// Builds the EML body (headers + HTML content).
/// `html_body` is already CRLF-normalized by `create_mail_html`, so it's appended as-is.
fn build_eml_content(subject: &str, html_body: &str) -> String {
    let mut eml_content = String::new();
    write!(
        eml_content,
        "Subject: {subject}\r\nX-Unsent: 1\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=utf-8\r\nContent-Transfer-Encoding: 8bit\r\n\r\n"
    )
    .expect("writing to a String never fails");

    eml_content.push_str(html_body);
    eml_content
}

pub fn launch_system_mailer(tasks: &[Task], image_data: &str, config: &Config) -> Result<()> {
    I18n::global().set_locale_from_config(config.email_locale);
    let html_text = create_mail_html(tasks, image_data)?;
    let subject_str = format!("%Y/%m/%d {}", fl!("task-status-report"));
    let subject = Zoned::now().date().strftime(&subject_str).to_string();

    let doc_dir = dirs::document_dir()
        .ok_or_else(|| anyhow!("Failed to get user document directory"))?
        .join("task-fighter-emails");
    fs::create_dir_all(&doc_dir)
        .context("Failed to create task-fighter-emails directory in Documents")?;

    let unique_id = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or_else(|_| "temp".to_string(), |d| d.as_millis().to_string());

    cleanup_old_eml_files(&doc_dir, EML_RETENTION_DAYS)?;

    let eml_path = doc_dir.join(format!("task_report_{unique_id}.eml"));
    info!("Mail file create at {}", eml_path.display());

    let eml_content = build_eml_content(&subject, &html_text);

    let mut temp_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&eml_path)
        .with_context(|| format!("Failed to create eml file at {}", eml_path.display()))?;

    temp_file
        .write_all(eml_content.as_bytes())
        .context("Failed to write eml content to file")?;
    temp_file
        .flush()
        .context("Failed to flush eml file to disk")?;

    drop(temp_file);

    open::that(eml_path).context("Failed to open eml file with the default mail client")?;

    Ok(())
}
