use anyhow::Result;
use scraper::{ElementRef, Html, Selector};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use serde_json::json;
use url::Url;

use scraperr_db::entities::job;
use scraperr_db::repositories::cron_job::CronJobRepository;
use scraperr_db::repositories::job::JobRepository;

// ---------------------------------------------------------------------------
// Selector abstraction: CSS or XPath (common subset translated to tree walk)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum SelectorKind {
    Css(Selector),
    Xpath(Vec<XpathStep>),
}

#[derive(Debug, Clone)]
struct XpathStep {
    axis: XpathAxis,
    tag: String,
    predicates: Vec<XpathPredicate>,
}

#[derive(Debug, Clone, PartialEq)]
enum XpathAxis {
    Child,
    Descendant,
}

#[derive(Debug, Clone)]
enum XpathPredicate {
    AttrEquals(String, String),
    AttrExists(String),
    Index(usize),
}

#[derive(Debug, Clone)]
enum Extraction {
    Text,
    Html,
    Attribute(String),
}

fn parse_selector(raw: &str) -> Option<(SelectorKind, Extraction)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with('/') || trimmed.starts_with("(//") {
        return parse_xpath(trimmed);
    }

    let (css_part, extraction) = if let Some(idx) = trimmed.find("::attr(") {
        let attr_part = &trimmed[idx + 7..];
        let attr = attr_part.trim_end_matches(')');
        (&trimmed[..idx], Extraction::Attribute(attr.to_string()))
    } else if let Some(idx) = trimmed.find("::html") {
        (&trimmed[..idx], Extraction::Html)
    } else if let Some(idx) = trimmed.find("::text") {
        (&trimmed[..idx], Extraction::Text)
    } else {
        (trimmed, Extraction::Text)
    };

    Selector::parse(css_part)
        .ok()
        .map(|s| (SelectorKind::Css(s), extraction))
}

fn parse_xpath(raw: &str) -> Option<(SelectorKind, Extraction)> {
    let s = raw.trim().trim_start_matches('(').trim_end_matches(')');

    let mut steps = Vec::new();
    let mut extraction = Extraction::Text;
    let mut remaining = s;

    while !remaining.is_empty() {
        let (axis, rest) = if remaining.starts_with("//") {
            (XpathAxis::Descendant, &remaining[2..])
        } else if remaining.starts_with('/') {
            (XpathAxis::Child, &remaining[1..])
        } else {
            break;
        };

        if rest.starts_with("text()") {
            extraction = Extraction::Text;
            remaining = &rest[6..];
            continue;
        }
        if rest.starts_with('@') {
            let attr_name: String = rest[1..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            extraction = Extraction::Attribute(attr_name);
            break;
        }

        let tag_end = rest
            .find(|c: char| c == '[' || c == '/' || c == '(')
            .unwrap_or(rest.len());
        let tag = if tag_end == 0 { "*" } else { &rest[..tag_end] };
        remaining = &rest[tag_end..];

        let mut predicates = Vec::new();
        while remaining.starts_with('[') {
            if let Some(close) = remaining.find(']') {
                let pred_str = &remaining[1..close];
                if let Some(pred) = parse_xpath_predicate(pred_str) {
                    predicates.push(pred);
                }
                remaining = &remaining[close + 1..];
            } else {
                break;
            }
        }

        steps.push(XpathStep {
            axis,
            tag: tag.to_string(),
            predicates,
        });
    }

    if steps.is_empty() {
        return None;
    }

    Some((SelectorKind::Xpath(steps), extraction))
}

fn parse_xpath_predicate(s: &str) -> Option<XpathPredicate> {
    let s = s.trim();
    if s.starts_with('@') {
        let inner = &s[1..];
        if let Some(eq_pos) = inner.find('=') {
            let attr = inner[..eq_pos].trim().to_string();
            let val = inner[eq_pos + 1..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            Some(XpathPredicate::AttrEquals(attr, val))
        } else {
            Some(XpathPredicate::AttrExists(inner.trim().to_string()))
        }
    } else if let Ok(n) = s.parse::<usize>() {
        Some(XpathPredicate::Index(n))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// XPath evaluation on scraper's tree
// ---------------------------------------------------------------------------

fn xpath_select<'a>(root: &'a Html, steps: &[XpathStep]) -> Vec<ElementRef<'a>> {
    let initial: Vec<ElementRef<'a>> = root
        .root_element()
        .children()
        .filter_map(ElementRef::wrap)
        .collect();
    let mut current_set = initial;

    for step in steps {
        let mut next_set: Vec<ElementRef<'a>> = Vec::new();

        for el in &current_set {
            let candidates: Vec<ElementRef<'a>> = match step.axis {
                XpathAxis::Child => el.children().filter_map(ElementRef::wrap).collect(),
                XpathAxis::Descendant => collect_descendants(*el),
            };

            for cand in candidates {
                if matches_tag(&cand, &step.tag) && matches_predicates(&cand, &step.predicates) {
                    next_set.push(cand);
                }
            }
        }

        for pred in &step.predicates {
            if let XpathPredicate::Index(idx) = pred {
                if *idx >= 1 && *idx <= next_set.len() {
                    next_set = vec![next_set[*idx - 1]];
                } else {
                    next_set.clear();
                }
            }
        }

        current_set = next_set;
    }

    current_set
}

fn collect_descendants<'a>(el: ElementRef<'a>) -> Vec<ElementRef<'a>> {
    let mut out = Vec::new();
    for child in el.children() {
        if let Some(child_el) = ElementRef::wrap(child) {
            out.push(child_el);
            out.extend(collect_descendants(child_el));
        }
    }
    out
}

fn matches_tag(el: &ElementRef, tag: &str) -> bool {
    tag == "*" || el.value().name() == tag
}

fn matches_predicates(el: &ElementRef, preds: &[XpathPredicate]) -> bool {
    for pred in preds {
        match pred {
            XpathPredicate::AttrEquals(attr, val) => {
                if el.value().attr(attr) != Some(val.as_str()) {
                    return false;
                }
            }
            XpathPredicate::AttrExists(attr) => {
                if el.value().attr(attr).is_none() {
                    return false;
                }
            }
            XpathPredicate::Index(_) => {}
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Extraction helpers
// ---------------------------------------------------------------------------

fn extract_value(el: &ElementRef, extraction: &Extraction) -> String {
    match extraction {
        Extraction::Text => el.text().collect::<Vec<_>>().join(" ").trim().to_string(),
        Extraction::Html => el.inner_html(),
        Extraction::Attribute(attr) => el.value().attr(attr).unwrap_or("").to_string(),
    }
}

// ---------------------------------------------------------------------------
// Element descriptor resolution
// ---------------------------------------------------------------------------

fn resolve_selectors_from_element(el: &serde_json::Value) -> Vec<(SelectorKind, Extraction)> {
    let mut out = Vec::new();

    if let Some(arr) = el.get("selectors").and_then(|v| v.as_array()) {
        for s in arr {
            if let Some(raw) = s.as_str() {
                if let Some(parsed) = parse_selector(raw) {
                    out.push(parsed);
                }
            }
        }
    }

    if let Some(raw) = el.get("selector").and_then(|v| v.as_str()) {
        if let Some(parsed) = parse_selector(raw) {
            out.push(parsed);
        }
    }

    if let Some(raw) = el.get("xpath").and_then(|v| v.as_str()) {
        if let Some(parsed) = parse_selector(raw) {
            out.push(parsed);
        }
    }

    out
}

fn extract_elements_from_doc(
    document: &Html,
    elements: &[serde_json::Value],
    base_url: &str,
) -> Vec<serde_json::Value> {
    let mut results: Vec<serde_json::Value> = Vec::new();

    for element in elements {
        let name = element
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let selectors = resolve_selectors_from_element(element);
        if selectors.is_empty() {
            continue;
        }

        let children = element.get("children").and_then(|v| v.as_array());

        let mut matched_nodes: Vec<ElementRef> = Vec::new();
        for (kind, _) in &selectors {
            match kind {
                SelectorKind::Css(sel) => {
                    matched_nodes.extend(document.select(sel));
                }
                SelectorKind::Xpath(steps) => {
                    matched_nodes.extend(xpath_select(document, steps));
                }
            }
        }

        let extraction = &selectors[0].1;

        for matched in &matched_nodes {
            let value = extract_value(matched, extraction);

            if let Some(child_defs) = children {
                let inner_html = matched.html();
                let inner_doc = Html::parse_fragment(&inner_html);
                let child_results = extract_elements_from_doc(&inner_doc, child_defs, base_url);

                let mut entry = serde_json::Map::new();
                entry.insert("text".to_string(), json!(value));
                entry.insert("children".to_string(), json!(child_results));
                results.push(json!({ name: entry }));
            } else {
                results.push(json!({
                    name: {
                        "text": value,
                    }
                }));
            }
        }
    }

    results
}

// ---------------------------------------------------------------------------
// HTTP client builder
// ---------------------------------------------------------------------------

fn build_client(job_options: &serde_json::Value) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (compatible; Scraperr/1.0)");

    if let Some(headers) = job_options.get("custom_headers").and_then(|v| v.as_object()) {
        let mut hm = reqwest::header::HeaderMap::new();
        for (k, v) in headers {
            if let Some(val) = v.as_str() {
                if let (Ok(name), Ok(value)) = (
                    reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                    reqwest::header::HeaderValue::from_str(val),
                ) {
                    hm.insert(name, value);
                }
            }
        }
        builder = builder.default_headers(hm);
    }

    builder.build().unwrap_or_else(|_| reqwest::Client::new())
}

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

fn find_next_page_url(
    document: &Html,
    pagination_selector: &str,
    current_url: &str,
) -> Option<String> {
    let (kind, _) = parse_selector(pagination_selector)?;
    let matched: Vec<ElementRef> = match &kind {
        SelectorKind::Css(sel) => document.select(sel).collect(),
        SelectorKind::Xpath(steps) => xpath_select(document, steps),
    };

    for el in matched {
        if let Some(href) = el.value().attr("href") {
            if let Ok(base) = Url::parse(current_url) {
                if let Ok(abs) = base.join(href) {
                    return Some(abs.to_string());
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Public API: execute a single job, poll for pending jobs, run cron scheduler
// ---------------------------------------------------------------------------

/// Execute a single scrape job by ID. Looks it up, runs it, updates status.
pub async fn execute_job_by_id(db: &DatabaseConnection, job_id: &str) -> Result<()> {
    let job = JobRepository::find_by_id(db, job_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Job not found: {job_id}"))?;
    execute_scrape_job(db, job).await
}

/// Poll for one queued job and execute it.
pub async fn process_pending_jobs(db: &DatabaseConnection) -> Result<()> {
    let pending = JobRepository::find_queued(db).await?;

    if let Some(j) = pending {
        let db = db.clone();
        tokio::spawn(async move {
            if let Err(e) = execute_scrape_job(&db, j).await {
                tracing::error!("Job execution error: {e}");
            }
        });
    }

    Ok(())
}

/// Background polling loop — call from a spawned task.
pub async fn run_polling_loop(db: DatabaseConnection, poll_interval_secs: u64) {
    tracing::info!("Worker polling loop started (interval={poll_interval_secs}s)");
    loop {
        if let Err(e) = process_pending_jobs(&db).await {
            tracing::error!("Error processing jobs: {e}");
        }
        tokio::time::sleep(std::time::Duration::from_secs(poll_interval_secs)).await;
    }
}

/// Cron scheduler — spawns scheduled jobs based on cron_jobs table.
pub async fn run_cron_scheduler(db: DatabaseConnection) -> Result<()> {
    use tokio_cron_scheduler::{Job, JobScheduler};

    let sched = JobScheduler::new().await?;

    let poll_db = db.clone();
    sched
        .add(Job::new_async("0 */1 * * * *", move |_uuid, _lock| {
            let db = poll_db.clone();
            Box::pin(async move {
                match CronJobRepository::find_all(&db).await {
                    Ok(crons) => {
                        for cron in crons {
                            tracing::debug!(
                                "Cron job: {} schedule={}",
                                cron.id,
                                cron.cron_expression
                            );
                        }
                    }
                    Err(e) => tracing::error!("Failed to load cron jobs: {e}"),
                }
            })
        })?)
        .await?;

    sched.start().await?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
}

// ---------------------------------------------------------------------------
// Internal: execute a single scrape job
// ---------------------------------------------------------------------------

async fn execute_scrape_job(db: &DatabaseConnection, j: job::Model) -> Result<()> {
    tracing::info!("Processing job {} for URL {}", j.id, j.url);

    let mut active: job::ActiveModel = j.clone().into();
    active.status = ActiveValue::Set("Scraping".to_string());
    active.update(db).await?;

    let job_options = j.job_options.clone().unwrap_or(json!({}));
    let client = build_client(&job_options);
    let elements = j.elements.as_array().cloned().unwrap_or_default();

    let multi_page = job_options
        .get("multi_page_scrape")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let pagination_selector = job_options
        .get("pagination_selector")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let max_pages: usize = job_options
        .get("max_pages")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;

    let mut all_results: Vec<serde_json::Value> = Vec::new();
    let mut current_url = j.url.clone();
    let mut pages_scraped = 0usize;

    loop {
        let response = match client.get(&current_url).send().await {
            Ok(r) => r,
            Err(e) => {
                mark_failed(db, &j.id, &format!("Request failed: {e}")).await?;
                return Ok(());
            }
        };

        let html_text = match response.text().await {
            Ok(t) => t,
            Err(e) => {
                mark_failed(db, &j.id, &format!("Failed to read body: {e}")).await?;
                return Ok(());
            }
        };

        let document = Html::parse_document(&html_text);
        let page_results = extract_elements_from_doc(&document, &elements, &current_url);

        let mut page_map = serde_json::Map::new();
        page_map.insert(current_url.clone(), json!(page_results));
        all_results.push(json!(page_map));

        pages_scraped += 1;

        if multi_page && !pagination_selector.is_empty() && pages_scraped < max_pages {
            if let Some(next_url) =
                find_next_page_url(&document, pagination_selector, &current_url)
            {
                if next_url != current_url {
                    current_url = next_url;
                    continue;
                }
            }
        }
        break;
    }

    let mut active: job::ActiveModel = j.into();
    active.status = ActiveValue::Set("Completed".to_string());
    active.result = ActiveValue::Set(json!(all_results));
    active.update(db).await?;

    Ok(())
}

async fn mark_failed(db: &DatabaseConnection, job_id: &str, msg: &str) -> Result<()> {
    let job = JobRepository::find_by_id(db, job_id).await?;
    if let Some(j) = job {
        let mut active: job::ActiveModel = j.into();
        active.status = ActiveValue::Set("Failed".to_string());
        active.result = ActiveValue::Set(json!({ "error": msg }));
        active.update(db).await?;
    }
    Ok(())
}
