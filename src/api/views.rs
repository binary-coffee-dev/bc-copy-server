use rocket_dyn_templates::{Template, context};

#[get("/")]
pub fn views() -> Template {
    Template::render("index", context! { value: "asd"})
}

