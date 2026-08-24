use actix_web::{HttpResponse, get};
use askama::Template;

pub struct SegmentSummary {
    pub href: &'static str,
    pub headline: &'static str,
    pub description: &'static str,
}

pub fn summaries() -> Vec<SegmentSummary> {
    vec![
        SegmentSummary {
            href: "/shadow-it/",
            headline: "Shadow-IT",
            description: "Employees may try to sync to unvetted tools no matter what. option63 strips sensitive fields at the boundary — so those tools only ever see a limited, low-risk subset.",
        },
        SegmentSummary {
            href: "/personal/",
            headline: "Personal Privacy",
            description: "Keep sensitive relationships private without losing convenience — redact names, addresses, and notes before they ever sync to the cloud or a low-trust device.",
        },
        SegmentSummary {
            href: "/ai/",
            headline: "AI Integrations",
            description: "Wire AI into your systems without handing it everything. option63 reduces each record to the minimal safe subset before payloads leave your boundary.",
        },
    ]
}

#[derive(Template)]
#[template(path = "segments_page.html")]
struct SegmentPageTemplate<'a> {
    title: &'a str,
    description: &'a str,
    eyebrow: &'a str,
    hero_headline: &'a str,
    hero_subhead: &'a str,
    problem_title: &'a str,
    problem_before: &'a str,
    problem_after: &'a str,
    demo_title: &'a str,
    demo_subhead: &'a str,
    demo_mount: &'a str,
    demo_js: &'a str,
    cta_primary: &'a str,
}

#[get("/shadow-it/")]
pub async fn it() -> HttpResponse {
    render_segment(SegmentCopy {
        title: "IT Security — option63 | option63.eu",
        description: "Limit what unvetted sync tools can expose. option63 strips sensitive contact fields at the boundary so shadow-IT carries far less risk to the company.",
        eyebrow: "IT Security",
        hero_headline: "De-risk shadow-IT",
        hero_subhead: "Employees may try to install unvetted sync tools, no matter what policy you write. option63 gives them a convenient and safe-ish alternative: by stripping sensitive fields before data reaches unvetted devices and apps, any tool they use — official or not — only ever sees a limited, low-risk subset.",
        problem_title: "The compliance gap",
        problem_before: "An employee syncs corporate contacts to a personal device or third-party app. The full vCard goes over — NOTE fields with client details, personal EMAIL/ADR, internal metadata. When an unvetted tool leaks, so does all of it.",
        problem_after: "option63 applies a property allow-list at the boundary. Whatever tool an employee uses only receives FN and TEL — never NOTE, ADR, EMAIL, or custom fields. If that tool leaks, there's nothing sensitive left for the company to lose.",
        demo_title: "See the transform live",
        demo_subhead: "Pulling a synthetic corporate CardDAV address book and applying the option63 property filter in your browser.",
        demo_mount: "shadow-it-demo",
        demo_js: "static/js/components/shadow-it-demo.js",
        cta_primary: "Contact us",
    })
}

#[get("/personal/")]
pub async fn personal() -> HttpResponse {
    render_segment(SegmentCopy {
        title: "Personal Privacy — option63 | option63.eu",
        description: "Take control of your own contact data and keep sensitive relationships private without losing convenience.",
        eyebrow: "Personal Privacy",
        hero_headline: "Your address book, on your terms",
        hero_subhead: "Cloud contacts may expose who you know and what you're dealing with. option63 transforms your own vCards before they ever sync — initials instead of full names, relayed emails instead of real addresses, redacted notes.",
        problem_title: "What your cloud learns about you",
        problem_before: "Your personal contacts sync to the cloud and AI agents in full: real names, personal email addresses, private notes about family, doctors, and sensitive relationships. A compromised account exposes all of it.",
        problem_after: "option63 rewrites the export: initials instead of a full name, no private EMAIL or ADR, notes removed. You still call and email the way you always have — but the metadata trail no longer points anywhere.",
        demo_title: "See the transform live",
        demo_subhead: "Pulling a synthetic personal address book and applying option63 pseudonymization and stripping in your browser.",
        demo_mount: "personal-demo",
        demo_js: "static/js/components/personal-demo.js",
        cta_primary: "Contact us",
    })
}

#[get("/ai/")]
pub async fn ai_integrator() -> HttpResponse {
    render_segment(SegmentCopy {
        title: "AI Integrations — option63 | option63.eu",
        description: "Wire AI into your systems without handing it everything — strip and redact data at the boundary before ingestion.",
        eyebrow: "AI Integrations",
        hero_headline: "Connect AI without leaking the whole dataset",
        hero_subhead: "Every connector you build to feed AI a system needs data through a boundary you can't fully trust. option63 reduces each ingested vCard to the minimum safe subset — the AI gets what it needs to work, never the raw contact store.",
        problem_title: "The ingestion boundary",
        problem_before: "A connector exports full contact records — names, personal emails, notes, addresses — straight into an AI service you don't control. Training data, logs, and retention policies are now out of your hands.",
        problem_after: "option63 sits at the ingestion point and applies an allow-list before the payload leaves. The model receives only the fields it needs for the task; identity-bearing and sensitive properties are stripped server-side.",
        demo_title: "See the transform live",
        demo_subhead: "Pulling a synthetic address book and reducing it to the minimal safe subset for AI ingestion, in your browser.",
        demo_mount: "ai-demo",
        demo_js: "static/js/components/ai-demo.js",
        cta_primary: "Contact us",
    })
}

struct SegmentCopy<'a> {
    title: &'a str,
    description: &'a str,
    eyebrow: &'a str,
    hero_headline: &'a str,
    hero_subhead: &'a str,
    problem_title: &'a str,
    problem_before: &'a str,
    problem_after: &'a str,
    demo_title: &'a str,
    demo_subhead: &'a str,
    demo_mount: &'a str,
    demo_js: &'a str,
    cta_primary: &'a str,
}

fn render_segment(c: SegmentCopy<'_>) -> HttpResponse {
    let html = SegmentPageTemplate {
        title: c.title,
        description: c.description,
        eyebrow: c.eyebrow,
        hero_headline: c.hero_headline,
        hero_subhead: c.hero_subhead,
        problem_title: c.problem_title,
        problem_before: c.problem_before,
        problem_after: c.problem_after,
        demo_title: c.demo_title,
        demo_subhead: c.demo_subhead,
        demo_mount: c.demo_mount,
        demo_js: c.demo_js,
        cta_primary: c.cta_primary,
    }
    .render()
    .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
