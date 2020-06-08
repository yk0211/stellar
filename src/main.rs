use actix::prelude::*;
use actix_files::Files;
use actix_web::{web, App, HttpServer, HttpResponse, middleware::Logger};

use diesel::{r2d2::ConnectionManager, MysqlConnection};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use config::{FileFormat, File, Config};
use log4rs;

mod models;
use models::DbExcutor;
#[allow(dead_code)]
struct AppState {
    db: Addr<DbExcutor>
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut config = Config::default();
    config.merge(File::new("config", FileFormat::Toml)).expect("There are not find config file");
   
    let db_url = config.get_str("mysql.db_url").unwrap();
    let bind_url = config.get_str("web.bind_url").unwrap();
    let log4rs_path = config.get_str("log.log4rs_path").unwrap();
    log4rs::init_file(log4rs_path, Default::default()).unwrap();

    let manager = ConnectionManager::<MysqlConnection>::new(db_url);
    let pool = r2d2::Pool::builder().build(manager).unwrap();
    let address = SyncArbiter::start(4, move || DbExcutor(pool.clone()));    

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(move || { 
        App::new()
            .data(AppState { db: address.clone() })
            .wrap(Logger::default())
            .service(Files::new("/public", "./public").show_files_listing().use_last_modified(true))
            .service(web::scope("/users").route("/show", web::get().to(index)))
    })
    .bind_openssl(bind_url, builder)?
    .run()
    .await
}

async fn index() -> HttpResponse {
    HttpResponse::Ok().body("test....")
}