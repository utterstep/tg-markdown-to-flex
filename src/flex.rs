use serde::Serialize;

/// Top-level Flex Message envelope.
#[derive(Debug, Clone, Serialize)]
pub struct FlexMessage {
    #[serde(rename = "type")]
    pub type_: FlexMessageType,
    #[serde(rename = "altText")]
    pub alt_text: String,
    pub contents: Bubble,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum FlexMessageType {
    #[serde(rename = "flex")]
    Flex,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bubble {
    #[serde(rename = "type")]
    pub type_: BubbleType,
    pub body: FlexBox,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<FlexBox>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum BubbleType {
    #[serde(rename = "bubble")]
    Bubble,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlexBox {
    #[serde(rename = "type")]
    pub type_: FlexBoxType,
    pub layout: BoxLayout,
    pub contents: Vec<Component>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<String>,
    #[serde(rename = "backgroundColor", skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum FlexBoxType {
    #[serde(rename = "box")]
    Box,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum BoxLayout {
    #[serde(rename = "vertical")]
    Vertical,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Component {
    #[serde(rename = "text")]
    Text { wrap: bool, contents: Vec<Span> },
    #[serde(rename = "button")]
    Button {
        action: UriAction,
        style: ButtonStyle,
        height: ButtonHeight,
    },
    #[serde(rename = "separator")]
    Separator {},
    #[serde(untagged)]
    Box(FlexBox),
}

#[derive(Debug, Clone, Serialize)]
pub struct Span {
    #[serde(rename = "type")]
    pub type_: SpanType,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<FontWeight>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<FontStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decoration: Option<TextDecoration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum SpanType {
    #[serde(rename = "span")]
    Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FontWeight {
    #[serde(rename = "bold")]
    Bold,
    #[serde(rename = "regular")]
    Regular,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FontStyle {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "italic")]
    Italic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TextDecoration {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "underline")]
    Underline,
    #[serde(rename = "line-through")]
    LineThrough,
}

#[derive(Debug, Clone, Serialize)]
pub struct UriAction {
    #[serde(rename = "type")]
    pub type_: UriActionType,
    pub label: String,
    pub uri: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum UriActionType {
    #[serde(rename = "uri")]
    Uri,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum ButtonStyle {
    #[serde(rename = "link")]
    Link,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum ButtonHeight {
    #[serde(rename = "sm")]
    Sm,
}

impl Span {
    pub fn plain(text: String) -> Self {
        Self {
            type_: SpanType::Span,
            text,
            weight: None,
            style: None,
            decoration: None,
            color: None,
            size: None,
        }
    }
}
