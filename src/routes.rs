use crate::network;
use std::net::{ IpAddr };
use actix_web::{ web, Responder, HttpRequest, HttpResponse };
use handlebars::Handlebars;

#[get("/")]
async fn index(hb: web::Data<Handlebars<'_>>) 
-> HttpResponse {

    let data = json!({
        "name": "Handlebars"
    });
    let body = hb.render("index", &data).unwrap();

    HttpResponse::Ok().body(body)
}

#[get("/{user}/{data}")]
async fn user(hb: web::Data<Handlebars<'_>>, web::Path(info): web::Path<(String, String)>) 
-> HttpResponse {

    let data = json!({
        "user": info.0,
        "data": info.1
    });
    let body = hb.render("user", &data).unwrap();

    HttpResponse::Ok().body(body)
}

#[get("/getid")]
async fn getid(req: HttpRequest) 
-> impl Responder {

    match req.peer_addr().unwrap().ip() {
        IpAddr::V4(ip4) => {
            println!("ip: {:?}", ip4);
            let mac = network::get_mac_through_arp(ip4);
            mac.to_string()
        },
        IpAddr::V6(ip6) => format!("Expected ipv4, got ipv6"),
    }
}
