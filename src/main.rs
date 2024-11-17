use std::{env, sync::Arc};

use actix_web::{
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use dotenv::dotenv;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct EmailBody {
    name: String,
    email: String,
    message: String,
}

#[derive(Clone)]
struct SmtpConfig {
    smtp_server: String,
    smtp_user: String,
    smtp_password: String,
    default_receiver: String,
}

#[actix_web::post("/contact")]
async fn send_contact(
    email_json: web::Json<EmailBody>,
    config: web::Data<SmtpConfig>,
) -> impl Responder {
    let email_body = email_json.into_inner();

    let message = Message::builder()
        .from(config.smtp_user.parse().unwrap())
        .to(config.default_receiver.parse().unwrap())
        .subject(format!(
            "Contact Us Form Submission From {}",
            email_body.name
        ))
        .body(format!(
            "You have a message from {}, with the email {} and they said: {}",
            email_body.name, email_body.email, email_body.message
        ))
        .unwrap();

    let creds = Credentials::new(config.smtp_user.clone(), config.smtp_password.clone());

    let mailer = SmtpTransport::relay(&config.smtp_server)
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&message) {
        Ok(_) => HttpResponse::Ok().body("Email sent successfully"),
        Err(e) => {
            eprintln!("Could not send email {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server error occured")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8000;
    println!("Starting server on port {:?}", port);

    dotenv().ok();

    let smtp_server = env::var("SMTP_SERVER").expect("SMTP_SERVER must be set");
    let smtp_user = env::var("SMTP_USER").expect("SMTP_USER must be set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    let default_receiver = env::var("DEFAULT_RECEIVER").expect("RECEIVER must be set");

    let email_config = Arc::new(SmtpConfig {
        smtp_server,
        smtp_user,
        smtp_password,
        default_receiver,
    });

    HttpServer::new(move || {
        App::new()
            .service(send_contact)
            .app_data(web::Data::from(email_config.clone()))
    })
    .bind(("0.0.0.0", port))?
    .workers(2)
    .run()
    .await
}
