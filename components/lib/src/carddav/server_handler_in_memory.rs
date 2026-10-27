//! In-memory CardDAV server used for testing.

use crate::carddav::protocol::{
    AddressDataType, AddressbookMultiget, AddressbookQuery, CARDDAV_NS, CarddavReport, Filter,
    FilterTest, MatchType, ParamFilter, ParamFilterContent, PropFilter, PropFilterContent,
    TextMatch, address_data_response, parse_report, supported_address_data_property,
};
use crate::carddav::server::{CarddavHandler, carddav_capabilities};
use crate::vcard::VCard;
use crate::vcard::property::param::Param;
use crate::webdav::error::DavError;
use crate::webdav::resource::Resource;
use crate::webdav::server::{DavCapabilities, DavHandler, Response};
use crate::webdav::xml::Node;
use crate::webdav::{
    DAV_NS, Depth, Multistatus, PropResponse, Property, PropertyValue, Propfind, Proppatch,
    Propstat, QName, Status, SyncCollection, SyncLevel,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

const SYNC_TOKEN_PREFIX: &str = "https://option63.eu/ns/sync/";

struct Store {
    resources: HashMap<String, Resource>,
    revisions: HashMap<String, u64>,
    deleted_resources: HashMap<String, u64>,
    sync_counter: u64,
    /// Address book collection paths and their descriptions.
    addressbooks: HashMap<String, Option<String>>,
}

impl Store {
    fn new() -> Self {
        Self {
            resources: HashMap::new(),
            revisions: HashMap::new(),
            deleted_resources: HashMap::new(),
            sync_counter: 0,
            addressbooks: HashMap::new(),
        }
    }

    fn bump(&mut self) -> u64 {
        self.sync_counter += 1;
        self.sync_counter
    }

    fn current_sync_token(&self) -> String {
        format!("{SYNC_TOKEN_PREFIX}{}", self.sync_counter)
    }
}

fn parse_sync_counter(token: &str) -> Option<u64> {
    let suffix = token
        .strip_prefix(SYNC_TOKEN_PREFIX)
        .or_else(|| token.rsplit('/').next())?;
    suffix.parse().ok()
}

fn is_immediate_child(parent: &str, child: &str) -> bool {
    let prefix = if parent.ends_with('/') {
        parent.to_string()
    } else {
        format!("{parent}/")
    };
    if !child.starts_with(&prefix) {
        return false;
    }
    let remainder = &child[prefix.len()..];
    let trimmed = remainder.strip_suffix('/').unwrap_or(remainder);
    !trimmed.is_empty() && !trimmed.contains('/')
}

fn match_sync_scope(parent: &str, child: &str, level: SyncLevel) -> bool {
    match level {
        SyncLevel::One => is_immediate_child(parent, child),
        SyncLevel::Infinite => {
            let prefix = if parent.ends_with('/') {
                parent.to_string()
            } else {
                format!("{parent}/")
            };
            child.starts_with(&prefix)
        }
    }
}

/// An in-memory CardDAV handler for testing and reference.
///
/// Stores vCard resources in a `HashMap` and implements the full
/// [`DavHandler`] + [`CarddavHandler`] trait surface, including sync-collection,
/// addressbook-query, and addressbook-multiget.
///
/// Discovery is supported via a fixed principal URL (`/principals/user/`) and
/// address book home set (`/addressbooks/user/`).
#[derive(Clone)]
pub struct InMemoryCarddavHandler {
    store: Arc<Mutex<Store>>,
}

impl InMemoryCarddavHandler {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(Store::new())),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Store> {
        self.store.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Creates an address book collection at the given path.
    pub fn create_addressbook(&self, path: &str, description: Option<String>) {
        let mut store = self.lock();
        store.addressbooks.insert(path.to_string(), description);
    }
}

impl Default for InMemoryCarddavHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("not found")]
    NotFound,
    #[error("method not allowed")]
    MethodNotAllowed,
    #[error("precondition failed")]
    PreconditionFailed,
    #[error("not implemented")]
    NotImplemented,
}

impl From<HandlerError> for DavError {
    fn from(e: HandlerError) -> DavError {
        match e {
            HandlerError::NotFound => DavError::Invalid("not found".into()),
            HandlerError::MethodNotAllowed => DavError::Invalid("method not allowed".into()),
            HandlerError::PreconditionFailed => DavError::Invalid("precondition failed".into()),
            HandlerError::NotImplemented => DavError::Invalid("not implemented".into()),
        }
    }
}

fn card_response(path: &str, res: &Resource, include_address_data: bool) -> PropResponse {
    let mut props = vec![
        Property::new(QName::dav("resourcetype"), PropertyValue::Empty),
        Property::new(
            QName::dav("getetag"),
            PropertyValue::Text(res.etag.clone().unwrap_or_default()),
        ),
        Property::new(
            QName::dav("getcontenttype"),
            PropertyValue::Text(res.content_type.clone()),
        ),
    ];
    if include_address_data {
        let card_text = String::from_utf8_lossy(&res.body).into_owned();
        props.push(address_data_response(&card_text));
    }
    PropResponse::propstat(
        path,
        Propstat {
            prop: props,
            status: Status::ok(),
            response_description: None,
        },
    )
}

fn card_response_for_propfind(path: &str, res: &Resource, request: &Propfind) -> PropResponse {
    match request {
        Propfind::AllProp => {
            let props = vec![
                Property::new(QName::dav("resourcetype"), PropertyValue::Empty),
                Property::new(
                    QName::dav("getetag"),
                    PropertyValue::Text(res.etag.clone().unwrap_or_default()),
                ),
                Property::new(
                    QName::dav("getcontenttype"),
                    PropertyValue::Text(res.content_type.clone()),
                ),
            ];
            PropResponse::propstat(
                path,
                Propstat {
                    prop: props,
                    status: Status::ok(),
                    response_description: None,
                },
            )
        }
        Propfind::PropName => {
            let props = vec![
                Property::name_only(QName::dav("resourcetype")),
                Property::name_only(QName::dav("getetag")),
                Property::name_only(QName::dav("getcontenttype")),
                Property::name_only(QName::new(CARDDAV_NS, "address-data")),
            ];
            PropResponse::propstat(
                path,
                Propstat {
                    prop: props,
                    status: Status::ok(),
                    response_description: None,
                },
            )
        }
        Propfind::Prop { properties } => {
            let mut ok_props = Vec::new();
            let mut not_found_props = Vec::new();
            let card_text = String::from_utf8_lossy(&res.body);

            for req_prop in properties {
                if req_prop.is(DAV_NS, "resourcetype") {
                    ok_props.push(Property::new(
                        QName::dav("resourcetype"),
                        PropertyValue::Empty,
                    ));
                } else if req_prop.is(DAV_NS, "getetag") {
                    ok_props.push(Property::new(
                        QName::dav("getetag"),
                        PropertyValue::Text(res.etag.clone().unwrap_or_default()),
                    ));
                } else if req_prop.is(DAV_NS, "getcontenttype") {
                    ok_props.push(Property::new(
                        QName::dav("getcontenttype"),
                        PropertyValue::Text(res.content_type.clone()),
                    ));
                } else if req_prop.is(CARDDAV_NS, "address-data") {
                    ok_props.push(address_data_response(&card_text));
                } else {
                    not_found_props.push(Property::name_only(req_prop.name.clone()));
                }
            }

            let mut propstats = Vec::new();
            if !ok_props.is_empty() || not_found_props.is_empty() {
                propstats.push(Propstat {
                    prop: ok_props,
                    status: Status::ok(),
                    response_description: None,
                });
            }
            if !not_found_props.is_empty() {
                propstats.push(Propstat {
                    prop: not_found_props,
                    status: Status::not_found(),
                    response_description: None,
                });
            }
            PropResponse::with_propstats(path, propstats)
        }
    }
}

fn collection_response_for_propfind(path: &str, request: &Propfind, store: &Store) -> PropResponse {
    let is_addressbook = store.addressbooks.contains_key(path);
    let description = store.addressbooks.get(path).cloned().flatten();

    let rt_prop = if is_addressbook {
        Property::new(
            QName::dav("resourcetype"),
            PropertyValue::Raw(vec![
                Node::elem(QName::dav("collection"), vec![]),
                crate::carddav::protocol::addressbook_element(),
            ]),
        )
    } else {
        Property::new(
            QName::dav("resourcetype"),
            PropertyValue::Raw(vec![Node::elem(QName::dav("collection"), vec![])]),
        )
    };

    match request {
        Propfind::AllProp => {
            let mut props = vec![
                rt_prop,
                Property::new(
                    QName::dav("current-user-principal"),
                    PropertyValue::Href("/principals/user/".into()),
                ),
                Property::new(
                    QName::new(CARDDAV_NS, "addressbook-home-set"),
                    PropertyValue::Hrefs(vec!["/addressbooks/user/".into()]),
                ),
            ];
            if is_addressbook {
                props.push(supported_address_data_property(&[
                    AddressDataType::vcard3(),
                    AddressDataType::vcard4(),
                ]));
                if let Some(desc) = description {
                    props.push(Property::new(
                        QName::new(CARDDAV_NS, "addressbook-description"),
                        PropertyValue::Text(desc),
                    ));
                }
            }
            PropResponse::propstat(
                path,
                Propstat {
                    prop: props,
                    status: Status::ok(),
                    response_description: None,
                },
            )
        }
        Propfind::PropName => {
            let mut props = vec![
                Property::name_only(QName::dav("resourcetype")),
                Property::name_only(QName::dav("current-user-principal")),
                Property::name_only(QName::new(CARDDAV_NS, "addressbook-home-set")),
            ];
            if is_addressbook {
                props.push(Property::name_only(QName::new(
                    CARDDAV_NS,
                    "supported-address-data",
                )));
                props.push(Property::name_only(QName::new(
                    CARDDAV_NS,
                    "addressbook-description",
                )));
            }
            PropResponse::propstat(
                path,
                Propstat {
                    prop: props,
                    status: Status::ok(),
                    response_description: None,
                },
            )
        }
        Propfind::Prop { properties } => {
            let mut ok_props = Vec::new();
            let mut not_found_props = Vec::new();

            for req_prop in properties {
                if req_prop.is(DAV_NS, "resourcetype") {
                    ok_props.push(rt_prop.clone());
                } else if req_prop.is(DAV_NS, "current-user-principal") {
                    ok_props.push(Property::new(
                        QName::dav("current-user-principal"),
                        PropertyValue::Href("/principals/user/".into()),
                    ));
                } else if req_prop.is(CARDDAV_NS, "addressbook-home-set") {
                    ok_props.push(Property::new(
                        QName::new(CARDDAV_NS, "addressbook-home-set"),
                        PropertyValue::Hrefs(vec!["/addressbooks/user/".into()]),
                    ));
                } else if req_prop.is(CARDDAV_NS, "supported-address-data") && is_addressbook {
                    ok_props.push(supported_address_data_property(&[
                        AddressDataType::vcard3(),
                        AddressDataType::vcard4(),
                    ]));
                } else if req_prop.is(CARDDAV_NS, "addressbook-description") {
                    if let Some(desc) = &description {
                        ok_props.push(Property::new(
                            QName::new(CARDDAV_NS, "addressbook-description"),
                            PropertyValue::Text(desc.clone()),
                        ));
                    } else {
                        not_found_props.push(Property::name_only(req_prop.name.clone()));
                    }
                } else {
                    not_found_props.push(Property::name_only(req_prop.name.clone()));
                }
            }

            let mut propstats = Vec::new();
            if !ok_props.is_empty() || not_found_props.is_empty() {
                propstats.push(Propstat {
                    prop: ok_props,
                    status: Status::ok(),
                    response_description: None,
                });
            }
            if !not_found_props.is_empty() {
                propstats.push(Propstat {
                    prop: not_found_props,
                    status: Status::not_found(),
                    response_description: None,
                });
            }
            PropResponse::with_propstats(path, propstats)
        }
    }
}

fn matches_text(value: &str, tm: &TextMatch) -> bool {
    let m = match tm.match_type {
        MatchType::Equals => value.eq_ignore_ascii_case(&tm.value),
        MatchType::Contains => value.to_lowercase().contains(&tm.value.to_lowercase()),
        MatchType::StartsWith => value.to_lowercase().starts_with(&tm.value.to_lowercase()),
        MatchType::EndsWith => value.to_lowercase().ends_with(&tm.value.to_lowercase()),
    };
    if tm.negate_condition { !m } else { m }
}

fn matches_param_filter(params: &[Param], pf: &ParamFilter) -> bool {
    let target_name = pf.name.as_bytes();
    let matching_params: Vec<&Param> = params
        .iter()
        .filter(|p| p.name().eq_ignore_ascii_case(target_name))
        .collect();

    match &pf.content {
        ParamFilterContent::IsNotDefined => matching_params.is_empty(),
        ParamFilterContent::Match(None) => !matching_params.is_empty(),
        ParamFilterContent::Match(Some(tm)) => {
            if matching_params.is_empty() {
                return false;
            }
            matching_params.iter().any(|p| {
                p.values().iter().any(|v| {
                    let val_str = String::from_utf8_lossy(v);
                    matches_text(&val_str, tm)
                })
            })
        }
    }
}

fn matches_prop_filter(card: &VCard, pf: &PropFilter) -> bool {
    let prop_name = pf.name.as_bytes();
    let matching_lines: Vec<&crate::vcard::ContentLine> = card
        .content_lines()
        .iter()
        .filter(|cl| cl.property().name().eq_ignore_ascii_case(prop_name))
        .collect();

    match &pf.content {
        PropFilterContent::IsNotDefined => matching_lines.is_empty(),
        PropFilterContent::Match {
            text_matches,
            param_filters,
        } => {
            if matching_lines.is_empty() {
                return false;
            }
            let test_fn = match pf.test {
                FilterTest::AnyOf => {
                    |results: &[bool]| results.is_empty() || results.iter().any(|&r| r)
                }
                FilterTest::AllOf => |results: &[bool]| results.iter().all(|&r| r),
            };

            matching_lines.iter().any(|cl| {
                let prop = cl.property();
                let mut results = Vec::new();

                if !text_matches.is_empty() {
                    let prop_unescaped = prop
                        .value()
                        .fields()
                        .iter()
                        .map(|f| {
                            f.values()
                                .iter()
                                .map(|v| String::from_utf8_lossy(v.as_slice()).into_owned())
                                .collect::<Vec<_>>()
                                .join(",")
                        })
                        .collect::<Vec<_>>()
                        .join(";");

                    let field_values: Vec<String> = prop
                        .value()
                        .fields()
                        .iter()
                        .flat_map(|f| {
                            f.values()
                                .iter()
                                .map(|v| String::from_utf8_lossy(v.as_slice()).into_owned())
                        })
                        .collect();

                    for tm in text_matches {
                        let matched = matches_text(&prop_unescaped, tm)
                            || field_values.iter().any(|fv| matches_text(fv, tm));
                        results.push(matched);
                    }
                }

                for param_f in param_filters {
                    let matched = matches_param_filter(prop.params(), param_f);
                    results.push(matched);
                }

                if results.is_empty() {
                    true
                } else {
                    test_fn(&results)
                }
            })
        }
    }
}

fn matches_filter(card_bytes: &[u8], filter: &Filter) -> bool {
    if filter.prop_filters.is_empty() {
        return true;
    }
    let (_, card) = match VCard::parse(card_bytes, false) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let test_fn = match filter.test {
        FilterTest::AnyOf => |results: &[bool]| results.iter().any(|&r| r),
        FilterTest::AllOf => |results: &[bool]| results.iter().all(|&r| r),
    };

    let results: Vec<bool> = filter
        .prop_filters
        .iter()
        .map(|pf| matches_prop_filter(&card, pf))
        .collect();

    test_fn(&results)
}

impl DavHandler for InMemoryCarddavHandler {
    type Error = HandlerError;

    fn map_error(&self, err: &Self::Error) -> (u16, DavError) {
        match err {
            HandlerError::NotFound => (404, DavError::Invalid("not found".into())),
            HandlerError::MethodNotAllowed => (405, DavError::Invalid("method not allowed".into())),
            HandlerError::PreconditionFailed => {
                (412, DavError::Invalid("precondition failed".into()))
            }
            HandlerError::NotImplemented => (501, DavError::Invalid("not implemented".into())),
        }
    }

    fn capabilities(&self) -> DavCapabilities {
        carddav_capabilities()
    }

    async fn get(&self, path: &str) -> Result<Response, Self::Error> {
        let store = self.lock();
        store
            .resources
            .get(path)
            .cloned()
            .map(Response::Resource)
            .ok_or(HandlerError::NotFound)
    }

    async fn put(&self, path: &str, mut resource: Resource) -> Result<Response, Self::Error> {
        let mut store = self.lock();
        let existed = store.resources.contains_key(path);
        let rev = store.bump();
        if resource.etag.is_none() || resource.etag.as_deref() == Some("") {
            resource.etag = Some(format!("\"{rev}\""));
        }
        store.resources.insert(path.to_string(), resource);
        store.revisions.insert(path.to_string(), rev);
        store.deleted_resources.remove(path);
        Ok(Response::Status(if existed { 200 } else { 201 }))
    }

    async fn delete(&self, path: &str) -> Result<Response, Self::Error> {
        let mut store = self.lock();
        if store.resources.remove(path).is_some() {
            let rev = store.bump();
            store.revisions.remove(path);
            store.deleted_resources.insert(path.to_string(), rev);
            Ok(Response::Status(204))
        } else {
            Err(HandlerError::NotFound)
        }
    }

    async fn propfind(
        &self,
        path: &str,
        depth: Depth,
        request: Propfind,
    ) -> Result<Response, Self::Error> {
        let store = self.lock();
        let mut responses = Vec::new();

        let is_virtual =
            path == "/" || path.starts_with("/principals/") || path.starts_with("/addressbooks/");

        if let Some(res) = store.resources.get(path) {
            responses.push(card_response_for_propfind(path, res, &request));
        } else if store.addressbooks.contains_key(path) || is_virtual {
            responses.push(collection_response_for_propfind(path, &request, &store));
        }

        if matches!(depth, Depth::One | Depth::Infinity) {
            let prefix = if path.ends_with('/') {
                path.to_string()
            } else {
                format!("{path}/")
            };
            let mut paths: Vec<&String> = store.resources.keys().collect();
            paths.extend(store.addressbooks.keys());
            paths.sort();
            paths.dedup();
            for p in paths {
                if p.as_str() == path {
                    continue;
                }
                let in_scope = match depth {
                    Depth::One => is_immediate_child(path, p),
                    Depth::Infinity => p.starts_with(&prefix),
                    Depth::Zero => false,
                };
                if !in_scope {
                    continue;
                }
                if let Some(res) = store.resources.get(p) {
                    responses.push(card_response_for_propfind(p, res, &request));
                } else if store.addressbooks.contains_key(p) {
                    responses.push(collection_response_for_propfind(p, &request, &store));
                }
            }
        }

        if responses.is_empty() {
            Err(HandlerError::NotFound)
        } else {
            Ok(Response::Multistatus(Multistatus {
                responses,
                ..Default::default()
            }))
        }
    }

    async fn proppatch(&self, path: &str, request: Proppatch) -> Result<Response, Self::Error> {
        let mut store = self.lock();
        let is_virtual =
            path == "/" || path.starts_with("/principals/") || path.starts_with("/addressbooks/");
        let exists = store.resources.contains_key(path)
            || store.addressbooks.contains_key(path)
            || is_virtual;
        if !exists {
            return Err(HandlerError::NotFound);
        }

        let mut props = Vec::new();
        for p in request.set {
            if p.is(CARDDAV_NS, "addressbook-description")
                && let PropertyValue::Text(desc) = &p.value
            {
                store
                    .addressbooks
                    .insert(path.to_string(), Some(desc.clone()));
            }
            props.push(Property::name_only(p.name));
        }
        for qname in request.remove {
            if qname.is(CARDDAV_NS, "addressbook-description") {
                store.addressbooks.insert(path.to_string(), None);
            }
            props.push(Property::name_only(qname));
        }

        let resp = PropResponse::propstat(
            path,
            Propstat {
                prop: props,
                status: Status::ok(),
                response_description: None,
            },
        );
        Ok(Response::Multistatus(Multistatus {
            responses: vec![resp],
            ..Default::default()
        }))
    }

    async fn mkcol(&self, path: &str, request: Option<Proppatch>) -> Result<Response, Self::Error> {
        let mut store = self.lock();
        if store.addressbooks.contains_key(path) || store.resources.contains_key(path) {
            return Err(HandlerError::MethodNotAllowed);
        }
        let mut description = None;
        if let Some(proppatch) = request {
            for p in proppatch.set {
                if p.is(CARDDAV_NS, "addressbook-description")
                    && let PropertyValue::Text(desc) = p.value
                {
                    description = Some(desc);
                }
            }
        }
        store.addressbooks.insert(path.to_string(), description);
        Ok(Response::Status(201))
    }

    async fn acl(&self, _path: &str, _acl: &crate::webdav::Acl) -> Result<Response, Self::Error> {
        Ok(Response::Status(200))
    }

    async fn report(&self, path: &str, depth: Depth, body: &[u8]) -> Result<Response, Self::Error> {
        if let Ok(sc) = SyncCollection::from_xml(body) {
            let store = self.lock();
            let mut responses = Vec::new();
            let propfind_req = if sc.prop.is_empty() {
                Propfind::AllProp
            } else {
                Propfind::Prop {
                    properties: sc.prop.clone(),
                }
            };

            match &sc.sync_token {
                None => {
                    let mut paths: Vec<&String> = store.resources.keys().collect();
                    paths.sort();
                    for p in paths {
                        if !match_sync_scope(path, p, sc.sync_level) {
                            continue;
                        }
                        let res = &store.resources[p];
                        responses.push(card_response_for_propfind(p, res, &propfind_req));
                    }
                }
                Some(token) if token.is_empty() => {
                    let mut paths: Vec<&String> = store.resources.keys().collect();
                    paths.sort();
                    for p in paths {
                        if !match_sync_scope(path, p, sc.sync_level) {
                            continue;
                        }
                        let res = &store.resources[p];
                        responses.push(card_response_for_propfind(p, res, &propfind_req));
                    }
                }
                Some(token) => {
                    let counter =
                        parse_sync_counter(token).ok_or(HandlerError::PreconditionFailed)?;
                    if counter > store.sync_counter {
                        return Err(HandlerError::PreconditionFailed);
                    }

                    let mut paths: Vec<&String> = store.resources.keys().collect();
                    paths.sort();
                    for p in paths {
                        if !match_sync_scope(path, p, sc.sync_level) {
                            continue;
                        }
                        let res = &store.resources[p];
                        let rev = store.revisions.get(p).copied().unwrap_or(0);
                        if rev > counter {
                            responses.push(card_response_for_propfind(p, res, &propfind_req));
                        }
                    }

                    let mut del_paths: Vec<&String> = store.deleted_resources.keys().collect();
                    del_paths.sort();
                    for p in del_paths {
                        if !match_sync_scope(path, p, sc.sync_level) {
                            continue;
                        }
                        let del_rev = store.deleted_resources[p];
                        if del_rev > counter {
                            responses
                                .push(PropResponse::with_status(p.clone(), Status::not_found()));
                        }
                    }
                }
            }
            let sync_token = store.current_sync_token();
            return Ok(Response::Multistatus(Multistatus {
                responses,
                sync_token: Some(sync_token),
                ..Default::default()
            }));
        }

        let report = parse_report(body).map_err(|_| HandlerError::NotImplemented)?;
        match report {
            CarddavReport::AddressbookQuery(query) => {
                self.addressbook_query(path, depth, query).await
            }
            CarddavReport::AddressbookMultiget(multiget) => {
                self.addressbook_multiget(path, multiget).await
            }
        }
    }
}

impl CarddavHandler for InMemoryCarddavHandler {
    async fn addressbook_query(
        &self,
        path: &str,
        depth: Depth,
        query: AddressbookQuery,
    ) -> Result<Response, Self::Error> {
        let store = self.lock();
        let mut responses: Vec<PropResponse> = Vec::new();

        let mut paths: Vec<&String> = store.resources.keys().collect();
        paths.sort();
        for p in paths {
            let res = &store.resources[p];
            if !is_immediate_child(path, p) && !matches!(depth, Depth::Infinity) {
                continue;
            }
            if !matches_filter(&res.body, &query.filter) {
                continue;
            }

            let resp = match &query.prop {
                Some(propfind) => {
                    let mut r = card_response_for_propfind(p, res, propfind);
                    if query.address_data.is_some() {
                        let has_address_data = r.propstats().is_some_and(|ps| {
                            ps.iter().any(|pst| {
                                pst.prop
                                    .iter()
                                    .any(|prop| prop.is(CARDDAV_NS, "address-data"))
                            })
                        });
                        if !has_address_data
                            && let Some(propstats) = r.propstats_mut()
                            && let Some(first_ps) = propstats.first_mut()
                        {
                            let card_text = String::from_utf8_lossy(&res.body).into_owned();
                            first_ps.prop.push(address_data_response(&card_text));
                        }
                    }
                    r
                }
                None => card_response(p, res, query.address_data.is_some()),
            };

            responses.push(resp);

            if let Some(limit) = query.limit
                && responses.len() >= limit as usize
            {
                break;
            }
        }

        Ok(Response::Multistatus(Multistatus {
            responses,
            ..Default::default()
        }))
    }

    async fn addressbook_multiget(
        &self,
        _path: &str,
        multiget: AddressbookMultiget,
    ) -> Result<Response, Self::Error> {
        let store = self.lock();

        let responses: Vec<PropResponse> = multiget
            .hrefs
            .iter()
            .map(|href| {
                if let Some(res) = store.resources.get(href) {
                    match &multiget.prop {
                        Some(propfind) => {
                            let mut r = card_response_for_propfind(href, res, propfind);
                            if multiget.address_data.is_some() {
                                let has_address_data = r.propstats().is_some_and(|ps| {
                                    ps.iter().any(|pst| {
                                        pst.prop
                                            .iter()
                                            .any(|prop| prop.is(CARDDAV_NS, "address-data"))
                                    })
                                });
                                if !has_address_data
                                    && let Some(propstats) = r.propstats_mut()
                                    && let Some(first_ps) = propstats.first_mut()
                                {
                                    let card_text = String::from_utf8_lossy(&res.body).into_owned();
                                    first_ps.prop.push(address_data_response(&card_text));
                                }
                            }
                            r
                        }
                        None => card_response(href, res, multiget.address_data.is_some()),
                    }
                } else {
                    PropResponse::with_status(href.clone(), Status::not_found())
                }
            })
            .collect();

        Ok(Response::Multistatus(Multistatus {
            responses,
            ..Default::default()
        }))
    }
}

impl InMemoryCarddavHandler {
    /// Get the current sync token (for test assertions).
    pub fn current_sync_token(&self) -> String {
        self.lock().current_sync_token()
    }
}
