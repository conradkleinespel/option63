use actix_web::{HttpResponse, get};
use askama::Template;

pub struct UnderConsideration {
    pub name: &'static str,
    pub description: &'static str,
}

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
            description: "Employees may try to sync to unvetted tools no matter what. option63 applies a property allow-list and drops named fields at the boundary - so those tools only ever see a limited, low-risk subset.",
        },
        SegmentSummary {
            href: "/personal/",
            headline: "Personal Privacy",
            description: "Keep sensitive relationships private without losing convenience - remove private fields with an allow-list and named drops before they sync to the cloud or a low-trust device.",
        },
        SegmentSummary {
            href: "/ai/",
            headline: "AI Integrations",
            description: "Wire AI into your systems without handing it everything. option63 reduces each record to the minimal safe subset with an allow-list before payloads leave your boundary.",
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
    demo_noscript: &'a str,
    commands: Vec<&'a str>,
    under_consideration: Vec<UnderConsideration>,
    cta_primary: &'a str,
}

#[get("/shadow-it/")]
pub async fn it() -> HttpResponse {
    render_segment(SegmentCopy {
        title: "IT Security - option63 | option63.eu",
        description: "Limit what unvetted sync tools can expose. option63 applies a property allow-list and drops named fields at the boundary so shadow-IT carries far less risk to the company.",
        eyebrow: "IT Security",
        hero_headline: "De-risk shadow-IT",
        hero_subhead: "Employees may try to install unvetted sync tools, no matter what policy you write. option63 gives them a convenient and safer alternative: apply an allow-list and drop named properties before data reaches unvetted devices and apps, so any tool they use - official or not - only ever sees a limited, low-risk subset.",
        problem_title: "The compliance gap",
        problem_before: "An employee syncs corporate contacts to a personal device or third-party app. The full vCard goes over - NOTE fields with client details, personal EMAIL/ADR, internal metadata. When an unvetted tool leaks, so does all of it.",
        problem_after: "option63 applies a property allow-list and can drop named properties at the boundary. Whatever tool an employee uses only receives the fields you allow - for example FN and TEL - never NOTE, ADR, EMAIL, or custom fields. If that tool leaks, there's nothing sensitive left for the company to lose.",
        demo_title: "See the allow-list live",
        demo_subhead: "A synthetic corporate vCard, reduced with option63's property allow-list in your browser.",
        demo_mount: "shadow-it-demo",
        demo_js: "static/js/components/shadow-it-demo.js",
        demo_noscript: "It would let you paste a vCard, choose properties to keep, and show the reduced card. The takeaway: an allow-list of FN and TEL removes email addresses, addresses, organization, title, and notes before the card leaves your boundary.",
        commands: vec![
            "o63 vcard show contacts.vcf --props FN,TEL",
            "o63 vcard drop contacts.vcf NOTE",
        ],
        under_consideration: vec![UnderConsideration {
            name: "CardDAV proxy filtering",
            description: "Apply the property allow-list and named drops inside the CardDAV proxy itself, so sync traffic is filtered on the server without changing the client.",
        }],
        cta_primary: "Contact us",
    })
}

#[get("/personal/")]
pub async fn personal() -> HttpResponse {
    render_segment(SegmentCopy {
        title: "Personal Privacy - option63 | option63.eu",
        description: "Take control of your own contact data and keep sensitive relationships private without losing convenience.",
        eyebrow: "Personal Privacy",
        hero_headline: "Your address book, on your terms",
        hero_subhead: "Cloud contacts may expose who you know and what you're dealing with. option63 applies a property allow-list and drops named properties on your own vCards before they sync.",
        problem_title: "What your cloud learns about you",
        problem_before: "Your personal contacts sync to the cloud and AI agents in full: real names, personal email addresses, private notes about family, doctors, and sensitive relationships. A compromised account exposes all of it.",
        problem_after: "option63 applies a property allow-list and drops named properties on the export: keep the fields you choose, drop EMAIL, ADR, or NOTE by name. You still have the contact you need, but private fields no longer leave your boundary.",
        demo_title: "See the allow-list live",
        demo_subhead: "A synthetic personal vCard, reduced with option63's property allow-list in your browser.",
        demo_mount: "personal-demo",
        demo_js: "static/js/components/personal-demo.js",
        demo_noscript: "It would let you paste a personal vCard, choose properties to keep, and show the reduced card. The takeaway: an allow-list of FN and TEL removes private emails, addresses, and notes before sync.",
        commands: vec![
            "o63 vcard show contacts.vcf --props FN,TEL",
            "o63 vcard drop contacts.vcf NOTE",
        ],
        under_consideration: vec![
            UnderConsideration {
                name: "Initials masking",
                description: "Replace names with initials while keeping enough identity for the contact to stay recognizable.",
            },
            UnderConsideration {
                name: "Email relay",
                description: "Swap real email addresses for relay addresses that forward mail, so the underlying address stays hidden.",
            },
            UnderConsideration {
                name: "Pseudonymization",
                description: "Replace identifying field values with stable aliases so records can still be matched without exposing who they describe.",
            },
        ],
        cta_primary: "Contact us",
    })
}

#[get("/ai/")]
pub async fn ai_integrator() -> HttpResponse {
    render_segment(SegmentCopy {
        title: "AI Integrations - option63 | option63.eu",
        description: "Wire AI into your systems without handing it everything - reduce each vCard to the minimal safe subset with an allow-list at the boundary.",
        eyebrow: "AI Integrations",
        hero_headline: "Connect AI without leaking the whole dataset",
        hero_subhead: "Every connector you build to feed AI a system needs data through a boundary you can't fully trust. option63 reduces each ingested vCard to the minimum safe subset with an allow-list - the AI gets what it needs to work, never the raw contact store.",
        problem_title: "The ingestion boundary",
        problem_before: "A connector exports full contact records - names, personal emails, notes, addresses - straight into an AI service you don't control. Training data, logs, and retention policies are now out of your hands.",
        problem_after: "option63 sits at the ingestion point and applies an allow-list before the payload leaves. The model receives only the fields it needs for the task; everything else is removed or dropped by name.",
        demo_title: "See the allow-list live",
        demo_subhead: "A synthetic address book record reduced to the minimal safe subset for AI ingestion with option63's property allow-list, in your browser.",
        demo_mount: "ai-demo",
        demo_js: "static/js/components/ai-demo.js",
        demo_noscript: "It would let you paste a vCard, choose properties to keep, and show the reduced card. The takeaway: an allow-list of FN and TITLE removes contact details and notes before AI ingestion.",
        commands: vec!["o63 vcard show contacts.vcf --props FN,TITLE"],
        under_consideration: vec![
            UnderConsideration {
                name: "Pseudonymization",
                description: "Replace identifying field values with stable aliases so the model can work with records without learning who they describe.",
            },
            UnderConsideration {
                name: "Redaction",
                description: "Remove sensitive field values entirely before contacts leave the boundary, leaving no trace of the original content.",
            },
        ],
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
    demo_noscript: &'a str,
    commands: Vec<&'a str>,
    under_consideration: Vec<UnderConsideration>,
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
        demo_noscript: c.demo_noscript,
        commands: c.commands,
        under_consideration: c.under_consideration,
        cta_primary: c.cta_primary,
    }
    .render()
    .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
