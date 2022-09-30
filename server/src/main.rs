/*
 * Author: Dylan Turner
 * Description:
 * - Entry point for database forwarding server
 * - End users != database users. We want to authenticate to do ANYTHING on the DB
 *   so we have a server to ask for the db info and IT accesses the db.
 *   This is that server
 */

mod db;

use rocket::{
    routes, build, get, launch, Config,
    config::LogLevel
};
use clap::{
    ArgMatches, command, arg
};
use crate::db::{
    register, login, request_password_reset, change_password,
    send_msg, recv_msg
};

const PORT: u16 = 9422;

static mut DB_USER: String = String::new();
static mut DB_PWORD: String = String::new();
static mut EMAIL_PWORD: String = String::new();

#[launch]
fn rocket() -> _ {
    let args = get_args();
    unsafe {
        DB_USER = args.get_one::<String>("USERNAME").unwrap().clone();
        DB_PWORD = args.get_one::<String>("PASSWORD").unwrap().clone();
        EMAIL_PWORD = args.get_one::<String>("EMAIL_PWORD").unwrap().clone();
    }

    // Just http, but not on a webpage, so who cares?
    let mut conf = Config::release_default();
    conf.port = PORT;
    conf.log_level = LogLevel::Debug;
    conf.address = "0.0.0.0".parse().unwrap();

    build().configure(conf).mount(
        "/", routes![
            register_user,
            login_user,
            request_reset_user_password,
            user_send_msg,
            user_recv_msg
        ]
    )
}

fn get_args() -> ArgMatches {
    command!()
        .about("Database server for simple web browser.")
        .arg(arg!(<USERNAME> "Server admin username").required(true))
        .arg(arg!(<PASSWORD> "Server admin password").required(true))
        .arg(arg!(<EMAIL_PWORD> "Password change request email password").required(true))
        .get_matches()
}

#[get("/register/<email>/<password>")]
async fn register_user(email: &str, password: &str) -> String {
    let (db_user, db_pword) = unsafe {
        (DB_USER.as_str(), DB_PWORD.as_str())
    };

    match register(email, password, db_user, db_pword).await {
        Ok(_) => String::from("success"),
        Err(err) => err.to_string()
    }
}

#[get("/login/<email>/<password>")]
async fn login_user(email: &str, password: &str) -> String {
    let (db_user, db_pword) = unsafe {
        (DB_USER.as_str(), DB_PWORD.as_str())
    };

    match login(email, password, db_user, db_pword).await {
        Ok(_) => String::from("success"),
        Err(err) => err.to_string()
    }
}

#[get("/req_pass_rst/<email>")]
async fn request_reset_user_password(email: &str) -> String {
    let (db_user, db_pword, email_pword) = unsafe {
        (DB_USER.as_str(), DB_PWORD.as_str(), EMAIL_PWORD.as_str())
    };

    match request_password_reset(email, db_user, db_pword, email_pword).await {
        Ok(_) => String::from("success"),
        Err(err) => err.to_string()
    }
}

#[get("/change_pass/<email>/<code>/<new_pass>")]
async fn change_user_password(email: &str, code: &str, new_pass: &str) -> String {
    let (db_user, db_pword) = unsafe {
        (DB_USER.as_str(), DB_PWORD.as_str())
    };

    match change_password(email, code, new_pass, db_user, db_pword).await {
        Ok(_) => String::from("success"),
        Err(err) => err.to_string()
    }
}

#[get("/send/<email>/<password>/<recp_email>/<msg>")]
async fn user_send_msg(email: &str, password: &str, recp_email: &str, msg: &str) -> String {
    let (db_user, db_pword) = unsafe {
        (DB_USER.as_str(), DB_PWORD.as_str())
    };

    match login(email, password, db_user, db_pword).await {
        Ok(_) => {},
        Err(err) => return err.to_string()
    }

    match send_msg(email, recp_email, msg, db_user, db_pword).await {
        Ok(_) => String::from("success"),
        Err(err) => err.to_string()
    }
}

#[get("/recv/<email>/<password>/<sender_email>")]
async fn user_recv_msg(email: &str, password: &str, sender_email: &str) -> String {
    let (db_user, db_pword) = unsafe {
        (DB_USER.as_str(), DB_PWORD.as_str())
    };

    match login(email, password, db_user, db_pword).await {
        Ok(_) => {},
        Err(err) => return err.to_string()
    }

    match recv_msg(sender_email, email, db_user, db_pword).await {
        Ok(msg) => String::from("success") + msg.as_str(),
        Err(err) => err.to_string()
    }
}

