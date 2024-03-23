use axum::response::Html;
use derive_builder::Builder;

#[derive(Clone, Builder, Default)]
pub struct Attributes {
    #[builder(setter(each(name = "htmx_attribute")), default = "vec![]")]
    pub htmx_attributes: Vec<HtmxAttributes>,
    #[builder(setter(each(name = "attribute")), default = "vec![]")]
    pub attributes: Vec<Attribute>,
}

fn html_element_with_attributes<'a>(component : &str, custom_attributes: Option<Vec<String>>, attributes: &Option<Attributes>) -> String {
    let mut component = format!("<{} ", component);
    if let Some(custom_attr) = custom_attributes {
        component += custom_attr.join(" ").as_str();
    }

    if let Some(a) = attributes {
        for h in a.htmx_attributes.iter() {
            component += format!(" {}", h.to_string().as_str()).as_str();
        }
        for a in a.attributes.iter() {
            component += format!(" {}", a.to_string().as_str()).as_str();
        }
    }
    component + " />"
}


#[derive(Clone)]
pub(crate) enum Attribute {
    CLASS(Vec<String>),
    WIDTH(u32),
    HEIGHT(u32),
    MARGIN(u32),
    WidthPercent(u32),
    HeightPercent(u32),
    WidthEm(u32),
    HeightEm(u32),
    WidthVw(u32),
    HeightVw(u32),
}

impl Attribute {
    fn to_string(&self) -> String {
        match self {
            Attribute::CLASS(s) => format!("class=\"{}\"", s.join(" ")),
            Attribute::WIDTH(u) => format!("width=\"{}\"", u),
            Attribute::HEIGHT(u) => format!("height=\"{}\"", u),
            Attribute::MARGIN(u) => format!("margin=\"{}\"", u),
            Attribute::WidthPercent(u) => format!("width=\"{}%\"", u),
            Attribute::HeightPercent(u) => format!("height=\"{}%\"", u),
            Attribute::WidthEm(u) => format!("width=\"{}em\"", u),
            Attribute::HeightEm(u) => format!("height=\"{}em\"", u),
            Attribute::WidthVw(u) => format!("width=\"{}vw\"", u),
            Attribute::HeightVw(u) => format!("height=\"{}vw\"", u),
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
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

impl<T>  From<T> for Box<dyn IntoHtml>
where T : IntoHtml + 'static
{
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

impl IntoHtml for String {
    fn html_string(&self) -> String {
        self.clone()
    }
}

impl IntoHtml for &str {
    fn html_string(&self) -> String {
        self.to_string()
    }
}

impl IntoHtml for &&str {
    fn html_string(&self) -> String {
        self.to_string()
    }
}

#[derive(Clone)]
pub struct Header(pub String);

impl IntoHtml for Header {
    fn html_string(&self) -> String {
        html_element_with_attributes("h1", None, &None) + self.0.as_str() + "</h1>"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct Div {
    #[builder(setter(each(name = "element", into)), default = "vec![]")]
    pub elements: Vec<Box<dyn IntoHtml>>,
    #[builder(setter(into, strip_option), default)]
    pub attributes: Option<Attributes>,
}

impl IntoHtml for Div {
    fn html_string(&self) -> String {
        let mut s = html_element_with_attributes("div", None, &self.attributes);
        for i in self.elements.iter() {
            s += i.html_string().as_str();
        }
        s += "</div>";
        s
    }
}

pub struct SimpleDiv<T:IntoHtml>(pub Option<Attributes>, pub T);

impl<T:IntoHtml> IntoHtml for SimpleDiv<T> {
    fn html_string(&self) -> String {
        html_element_with_attributes("div", None, &self.0) + self.1.html_string().as_str() + "</div>"
    }
}

pub struct Anchor<T: IntoHtml>(pub String, pub T);

impl<T: IntoHtml> IntoHtml for Anchor<T> {
    fn html_string(&self) -> String {
        html_element_with_attributes("a",
                                     Some(vec![
                                         format!("href=\"{}\"", self.0)]),
                                     &None)
            + self.1.html_string().as_str()
            + "</a>"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct Img {
    pub uri: String,
    pub alt_text: String,
    #[builder(setter(into, strip_option), default)]
    pub attributes: Option<Attributes>,
}

impl IntoHtml for Img {
    fn html_string(&self) -> String {
        html_element_with_attributes("img",
                                     Some(vec![format!("src=\"{}\"", self.uri.as_str()),
                                               format!("alt=\"{}\"", self.alt_text.as_str())]),
                                     &self.attributes)
        + "</img>"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct Ulist {
    #[builder(setter(each(name = "item", into)), default = "vec![]")]
    pub items: Vec<Box<dyn IntoHtml>>,
    #[builder(setter(into, strip_option), default)]
    pub attributes: Option<Attributes>,
    #[builder(setter(into, strip_option), default)]
    pub item_attributes: Option<Attributes>,
}


// for posterity this was completely AI generated. WTF
impl IntoHtml for Ulist{
    fn html_string(&self) -> String {
        let mut s = html_element_with_attributes("ul", None, &self.attributes);
        for i in self.items.iter() {
            s += html_element_with_attributes("li", None, &self.item_attributes).as_str();
            s += i.html_string().as_str();
            s += "</li>";
        }
        s + "</ul>"
    }
}

pub struct Hr;

impl IntoHtml for Hr {
    fn html_string(&self) -> String {
        html_element_with_attributes("hr", None, &None) + "</hr>"
    }
}

pub struct Header1(pub String);

impl IntoHtml for Header1 {
    fn html_string(&self) -> String {
        html_element_with_attributes("h1", None, &None) + self.0.as_str() + "</h1>"
    }
}

pub struct Header2(pub String);

impl IntoHtml for Header2 {
    fn html_string(&self) -> String {
        html_element_with_attributes("h2", None, &None) + self.0.as_str() + "</h2>"
    }
}
