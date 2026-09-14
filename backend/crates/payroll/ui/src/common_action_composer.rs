//! Shared SSR presentation over an already-authorized owner projection.
//! These controls do not validate authority, decode business values, or persist
//! drafts. The receiving owner must validate every token, address and operation.
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldAddress {
    pub field_id: String,
    pub parent_item_ids: Vec<String>,
    pub item_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Choice {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Debug)]
pub enum EditorValue {
    Text(String),
    Decimal(String),
    /// Acknowledged editor text, never a validated business value.
    Incomplete(String),
    Enum {
        selected: Option<String>,
        choices: Vec<Choice>,
    },
}

#[derive(Clone, Debug)]
pub struct AuthorizedField {
    pub address: FieldAddress,
    pub label: String,
    pub value: EditorValue,
    pub required: bool,
}

#[derive(Clone, Debug)]
pub struct FieldError {
    pub address: FieldAddress,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposerOperation {
    Save,
    Submit,
}

#[derive(Clone, Debug)]
pub struct MutationContext {
    pub action_path: String,
    pub csrf_token: String,
    pub command_id: String,
    pub expected_token: String,
    pub draft_token: String,
    pub operations: Vec<ComposerOperation>,
}

#[derive(Clone, Debug)]
pub struct ConflictRecovery {
    pub message: String,
    pub authorized_reload_path: String,
}

#[derive(Clone, Debug)]
pub struct AuthorizedComposer {
    pub title: String,
    pub fields: Vec<AuthorizedField>,
    pub errors: Vec<FieldError>,
    pub mutation: Option<MutationContext>,
    pub conflict: Option<ConflictRecovery>,
}

/// Hex segments and distinct parent/item markers preserve every boundary,
/// including empty strings. No row index or editable label becomes identity.
pub fn control_id(address: &FieldAddress) -> String {
    fn segment(out: &mut String, value: &str) {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for byte in value.bytes() {
            out.push(char::from(HEX[usize::from(byte >> 4)]));
            out.push(char::from(HEX[usize::from(byte & 15)]));
        }
    }
    let mut id = String::from("field_");
    segment(&mut id, &address.field_id);
    for parent in &address.parent_item_ids {
        id.push_str("-p-");
        segment(&mut id, parent);
    }
    if let Some(item) = &address.item_id {
        id.push_str("-i-");
        segment(&mut id, item);
    } else {
        id.push_str("-n");
    }
    id
}

pub fn control_name(address: &FieldAddress) -> String {
    control_id(address)
}

// Only server-composed UI paths are accepted. Escaping alone does not make a
// URL safe. No scheme, authority, percent-encoded traversal or dot segments.
fn local_ui_path(path: &str) -> bool {
    path.starts_with("/_ui/")
        && path.bytes().all(|b| {
            b.is_ascii_alphanumeric() || matches!(b, b'/' | b'_' | b'-' | b'?' | b'=' | b'&')
        })
}

fn field_control(field: &AuthorizedField, errors: &[FieldError]) -> AnyView {
    let id = control_id(&field.address);
    let messages = errors
        .iter()
        .filter(|e| e.address == field.address)
        .map(|e| e.message.as_str())
        .collect::<Vec<_>>()
        .join(" · ");
    let invalid = !messages.is_empty() || matches!(field.value, EditorValue::Incomplete(_));
    let error_id = format!("{id}-error");
    let described = (!messages.is_empty()).then(|| error_id.clone());
    let input = match field.value.clone() {
        EditorValue::Enum { selected, choices } => {
            let has_selection = choices
                .iter()
                .any(|choice| Some(&choice.value) == selected.as_ref());
            view! {
                <select id=id.clone() name=control_name(&field.address) required=field.required
                    aria-invalid=invalid.then_some("true") aria-describedby=described>
                    {(!field.required || !has_selection).then(|| view! {
                        <option value="" selected=!has_selection>
                            {if field.required { "선택해 주세요" } else { "선택 안 함" }}
                        </option>
                    })}
                    {choices.into_iter().map(|choice| {
                        let is_selected = Some(&choice.value) == selected.as_ref();
                        view! { <option value=choice.value selected=is_selected>{choice.label}</option> }
                    }).collect_view()}
                </select>
            }.into_any()
        }
        EditorValue::Text(value) | EditorValue::Decimal(value) | EditorValue::Incomplete(value) => {
            let inputmode = matches!(field.value, EditorValue::Decimal(_)).then_some("decimal");
            view! {
                <input id=id.clone() name=control_name(&field.address) type="text" value=value
                    inputmode=inputmode required=field.required
                    aria-invalid=invalid.then_some("true") aria-describedby=described/>
            }
            .into_any()
        }
    };
    view! {
        <div class="composer-field">
            <label for=id>{field.label.clone()}{field.required.then_some(" (필수)")}</label>
            {input}
            {(!messages.is_empty()).then(|| view! { <p class="field-error" id=error_id>{messages}</p> })}
        </div>
    }.into_any()
}

fn read_only_field(field: &AuthorizedField) -> AnyView {
    let text = match &field.value {
        EditorValue::Text(value) | EditorValue::Decimal(value) | EditorValue::Incomplete(value) => {
            value.clone()
        }
        EditorValue::Enum { selected, choices } => choices
            .iter()
            .find(|choice| Some(&choice.value) == selected.as_ref())
            .map_or_else(|| "선택되지 않음".to_owned(), |choice| choice.label.clone()),
    };
    view! {
        <div class="composer-field">
            <dt>{field.label.clone()}</dt><dd id=control_id(&field.address) tabindex="-1">{text}</dd>
        </div>
    }.into_any()
}

/// Render a standalone no-JS page. No scripts, hidden projection payloads,
/// fetched options or inferred permissions are added by this presentation.
pub fn render(model: &AuthorizedComposer) -> String {
    let mutation = model
        .mutation
        .as_ref()
        .filter(|context| local_ui_path(&context.action_path) && !context.operations.is_empty());
    // A returned error describes the previous input. In a no-JS form the user
    // must be able to correct it and retry; the owner validates the new input.
    // Stale revision recovery, unlike validation, requires a new context first.
    let blocked = model.conflict.is_some();
    let body = if let Some(context) = mutation {
        let save = context.operations.contains(&ComposerOperation::Save);
        let submit = context.operations.contains(&ComposerOperation::Submit);
        view! {
            <form method="post" action=context.action_path.clone()>
                <input type="hidden" name="csrf_token" value=context.csrf_token.clone()/>
                <input type="hidden" name="command_id" value=context.command_id.clone()/>
                <input type="hidden" name="expected_token" value=context.expected_token.clone()/>
                <input type="hidden" name="draft_token" value=context.draft_token.clone()/>
                {model.fields.iter().map(|field| field_control(field, &model.errors)).collect_view()}
                <div class="composer-actions">
                    {save.then(|| view! { <button type="submit" name="operation" value="save" formnovalidate>"임시 저장"</button> })}
                    {submit.then(|| view! { <button class="primary" type="submit" name="operation" value="submit" disabled=blocked>"제출"</button> })}
                </div>
                {blocked.then(|| view! { <p class="composer-hint">"입력 내용과 변경 사항을 확인한 뒤 제출할 수 있습니다."</p> })}
            </form>
        }.into_any()
    } else {
        view! { <dl>{model.fields.iter().map(read_only_field).collect_view()}</dl> }.into_any()
    };
    let summary = (!model.errors.is_empty()).then(|| view! {
        <section class="composer-notice" role="alert" tabindex="-1" autofocus aria-label="입력 확인">
            <h2>"입력 내용을 확인해 주세요"</h2>
            <ul>{model.errors.iter().map(|error| {
                let linked = model.fields.iter().any(|field| field.address == error.address);
                view! { <li>{if linked {
                    view! { <a href=format!("#{}", control_id(&error.address))>{error.message.clone()}</a> }.into_any()
                } else { error.message.clone().into_any() }}</li> }
            }).collect_view()}</ul>
        </section>
    });
    let conflict = model.conflict.as_ref().map(|conflict| {
        view! {
            <section class="composer-notice" role="status" aria-label="변경 확인">
                <p>{conflict.message.clone()}</p>
                {local_ui_path(&conflict.authorized_reload_path).then(|| view! {
                    <a href=conflict.authorized_reload_path.clone()>"현재 내용 확인"</a>
                })}
            </section>
        }
    });
    let mut html = String::from("<!DOCTYPE html>");
    html.push_str(&view! {
        <html lang="ko"><head>
            <meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/>
            <title>{model.title.clone()}</title>
            <style>{include_str!("common_action_composer.css")}</style>
        </head><body><main class="composer">
            <header><p class="composer-eyebrow">"Console"</p><h1>{model.title.clone()}</h1></header>
            {summary}{conflict}{body}
        </main></body></html>
    }.to_html());
    html
}
