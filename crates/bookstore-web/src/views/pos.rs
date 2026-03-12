use askama::Template;

#[derive(Template)]
#[template(path = "pos/shell.html")]
pub struct PosShellTemplate {}
