extern crate env_logger;
#[macro_use]
extern crate tower_web;

use tower_web::ServiceBuilder;
use tower_web::view::Handlebars;

/// This type will be the web service implementation.
#[derive(Clone, Debug)]
struct HtmlResource;

#[derive(Debug, Response)]
struct MyResponse {
    title: &'static str,
}

impl_web! {
    impl HtmlResource {
        // Respond as HTML. For this to work, a serializer supporting HTML must
        // be added to the service.
        //
        // If no serializer is specified, a 500 response will be returned.
        //
        #[get("/")]
        #[content_type("html")]
        #[web(template = "templates/index")]
        fn hello_world(&self) -> Result<MyResponse, ()> {
            Ok(MyResponse {
                title: "Handler variable",
            })
        }
    }
}

pub fn main() {
    let _ = env_logger::try_init();

    let addr = "10.0.1.1:80".parse().expect("Invalid address");
    println!("Listening on http://{}", addr);

    ServiceBuilder::new()
        .resource(HtmlResource)
        // Add the handlebars serializer to the application. This uses the
        // template rendering default settings. Templates are located at
        // the crate root in the `templates` directory. Template files
        // use the `.hbs` extension.
        //
        // The handlebars serializer is configured by calling
        // `Handlebars::new_with_registry` and passing in a configured
        // registry. This allows changing the template directory as well
        // as defining helpers and other configuration options.
        //
        // See the `handlebars` crate for more documentation on configuration
        // options.
        .serializer(Handlebars::new())
        .run(&addr)
        .unwrap();
}
