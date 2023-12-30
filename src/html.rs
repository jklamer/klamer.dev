use axum::response::Html;
use derive_builder::Builder;

pub fn html_component<'a>(component : &str, properties: &[impl AsRef<str>]) -> String {
    let mut component = "<".to_string() + component + " ";
    for p in properties.iter() {
        component += (*p).as_ref();
    }
    component + " />"
}

#[derive(Clone, Builder, Default)]
pub struct Attributes {
    #[builder(setter(each(name = "htmx_attribute")), default = "vec![]")]
    pub htmx_attributes: Vec<HtmxAttributes>,
    #[builder(setter(each(name = "attribute")), default = "vec![]")]
    pub attributes: Vec<Attribute>,
}


#[derive(Clone)]
pub(crate) enum Attribute {
    HREF(String),
}

impl Attribute {
    fn to_string(&self) -> String {
        match self {
            Attribute::HREF(s) => "href=\"".to_string() + s + "\"",
        }
    }
}

#[derive(Clone)]
pub(crate) enum HtmxAttributes {
    GET(String),
    POST(String),
    PUT(String),
    PATCH(String),
    DELETE(String),
}

impl HtmxAttributes {
    fn to_string(&self) -> String {
        match self {
            HtmxAttributes::GET(s) => "hx-get=\"".to_string() + s + "\"",
            HtmxAttributes::POST(s) => "hx-post=\"".to_string() + s + "\"",
            HtmxAttributes::PUT(s) => "hx-put=\"".to_string() + s + "\"",
            HtmxAttributes::PATCH(s) => "hx-patch=\"".to_string() + s + "\"",
            HtmxAttributes::DELETE(s) => "hx-delete=\"".to_string() + s + "\"",
        }
    }
}

pub trait IntoHtml {
    fn html_response(&self) -> Html<String> {
        Html(self.html_string())
    }
    fn html_string(&self) -> String;
}


#[derive(Clone)]
pub struct Header(pub String);

impl IntoHtml for Header {
    fn html_string(&self) -> String {
        html_component("h1", &[" "]) + self.0.as_str() + "</h1>"
    }
}

pub struct Div {
    pub components: Vec<Box<dyn IntoHtml>>,
    pub attributes: Attributes,
}

impl IntoHtml for Div {
    fn html_string(&self) -> String {
        let mut s = html_component("div", &self.attributes.htmx_attributes.iter().map(|x| x.to_string())
            .chain(self.attributes.attributes.iter().map(|x| x.to_string()))
            .collect::<Vec<String>>().as_slice());
        for i in self.components.iter() {
            s += i.html_string().as_str();
        }
        s += "</div>";
        s
    }
}

pub struct Anchor<T: IntoHtml>(pub String, pub T);

impl<T: IntoHtml> IntoHtml for Anchor<T> {
    fn html_string(&self) -> String {
        html_component("a",
                       &["href=\"".to_string() + self.0.as_str() + "\"",
                        "style=\"text-decoration:none;color:inherit;\"".to_string()])
            .to_string()
            + self.1.html_string().as_str()
            + "</a>"
    }
}

pub struct Img {
    pub uri: String,
    pub alt_text: String,
}

impl IntoHtml for Img {
    fn html_string(&self) -> String {
        html_component("img",
                       &["src=\"".to_string() + self.uri.as_str() + "\"",
                                "alt=\"".to_string() + self.alt_text.as_str() + "\""])
        + "</img>"
    }
}
