/*
 * Author: Dylan Turner
 * Description: Handle all database control for the application
 */

use std::{
    hash::{
        Hash, Hasher
    }, collections::hash_map::DefaultHasher,
    error::Error
};
use mongodb::{
    Client, Collection,
    bson::doc
};
use serde::{
    Serialize, Deserialize
};
use rand::{
    Rng, thread_rng
};
use reqwest::get;
use lettre_email::EmailBuilder;
use lettre::{
    smtp::authentication::Credentials,
    Transport, SmtpClient
};

// This gets passed in at runtime
const DB_LOGIN: [&'static str; 3] = [
    "mongodb://",
    ":",
    "@127.0.0.1:27017"
];

// For sending email code
const SALT_LEN: usize = 64;
const DB: &'static str = "forset";
const USER_COLL: &'static str = "users";
const PWORD_CHANGE_COLL: &'static str = "password_change_requests";
const MSG_COLL: &'static str = "messages";

// For sending email code
const EMAIL_EMAIL: &'static str = "vid.rooms.no.reply@gmail.com";
const EMAIL_SMTP: &'static str = "smtp.gmail.com";

#[derive(Serialize, Deserialize)]
struct User {
    pub email: String,
    pub pword_salt: String,
    pub pword_hash: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct PasswordChangeRequest {
    pub user_email: String,
    pub code: String
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub sender_email: String,
    pub recp_email: String,
    pub content: String
}

impl Default for Message {
    fn default() -> Self {
        Self {
            sender_email: String::new(),
            recp_email: String::new(),
            content: String::new()
        }
    }
}

/* Exposed functions */

pub async fn send_msg(
        sender_email: &str, recp_email: &str, content: &str,
        db_user: &str, db_pword: &str) -> Result<(), Box<dyn Error>> {
    let db_login = String::from(DB_LOGIN[0]) + db_user + DB_LOGIN[1] + db_pword + DB_LOGIN[2];

    let client = Client::with_uri_str(db_login).await?;
    let db = client.database(DB);
    let messages: Collection<Message> = db.collection(MSG_COLL);

    // Delete all first
    let filter = doc! {
        "sender_email": sender_email,
        "recp_email": recp_email
    };
    messages.delete_many(filter, None).await?;

    // The new one
    messages.insert_one(
        Message {
            sender_email: String::from(sender_email),
            recp_email: String::from(recp_email),
            content: String::from(content)
        }, None
    ).await?;

    Ok(())
}

pub async fn recv_msg(
        sender_email: &str, recp_email: &str,
        db_user: &str, db_pword: &str) -> Result<String, Box<dyn Error>> {
    let db_login = String::from(DB_LOGIN[0]) + db_user + DB_LOGIN[1] + db_pword + DB_LOGIN[2];

    let client = Client::with_uri_str(db_login).await?;
    let db = client.database(DB);
    let messages: Collection<Message> = db.collection(MSG_COLL);

    let filter = doc! {
        "sender_email": sender_email,
        "recp_email": recp_email
    };
    match messages.find_one(filter, None).await? {
        Some(msg) => Ok(msg.content),
        None => Err("No message found!".into())
    }
}

// Login (throws an error with wrong details)
pub async fn login(
        email_txt: &str, pword_txt: &str,
        db_user: &str, db_pword: &str) -> Result<(), Box<dyn Error>> {
    let db_login = String::from(DB_LOGIN[0]) + db_user + DB_LOGIN[1] + db_pword + DB_LOGIN[2];

    let client = Client::with_uri_str(db_login).await?;
    let db = client.database(DB);
    let users: Collection<User> = db.collection(USER_COLL);

    let filter = doc! {
        "email": email_txt        
    };
    let result = users.find_one(filter, None).await?;
    
    let user = match result {
        Some(ref u) => u,

        // Give generic answer for security!
        None => Err("Incorrect email/password combo.")?
    };

    let salt_and_pass = user.pword_salt.clone() + pword_txt;
    let test_hash = get_hash(&salt_and_pass);

    if test_hash == user.pword_hash {
        // Delete all messages on login to make sure always gets cleaned up
        let messages: Collection<Message> = db.collection(MSG_COLL);
        let filter = doc! {
            "sender_email": email_txt
        };
        messages.delete_many(filter, None).await?;
        let filter = doc! {
            "recp_email": email_txt
        };
        messages.delete_many(filter, None).await?;

        Ok(())
    } else {
        Err("Incorrect email/password combo.")?
    }
}

// Register a new user
pub async fn register(
        email_txt: &str, pword_txt: &str,
        db_user: &str, db_pword: &str) -> Result<(), Box<dyn Error>> {
    assert_valid_email(email_txt).await?;

    let db_login = String::from(DB_LOGIN[0]) + db_user + DB_LOGIN[1] + db_pword + DB_LOGIN[2];
    
    // First do a check to see if the user exists
    let client = Client::with_uri_str(db_login).await?;
    let db = client.database(DB);
    let users: Collection<User> = db.collection(USER_COLL);
    assert_unused_email(email_txt, &users).await?;

    let salt = get_random_salt();
    let salt_and_pass = salt.clone() + pword_txt;
    let hash = get_hash(&salt_and_pass.to_string());

    let new_user = User {
        email: String::from(email_txt.clone()),
        pword_salt: salt,
        pword_hash: hash
    };
    users.insert_one(new_user, None).await?;

    Ok(())
}

// Send a password reset request for the user
pub async fn request_password_reset(
        email_txt: &str,
        db_user: &str, db_pword: &str, email_pword: &str) -> Result<(), Box<dyn Error>> {
    let db_login = String::from(DB_LOGIN[0]) + db_user + DB_LOGIN[1] + db_pword + DB_LOGIN[2];

    let client = Client::with_uri_str(db_login).await?;
    let vr_db = client.database(DB);
    let reset_requests: Collection<PasswordChangeRequest> = vr_db.collection(PWORD_CHANGE_COLL);

    // Create code for reset request and send it to the user
    let code = get_random_salt();
    send_reset_email(email_txt, code.clone().as_str(), email_pword).await?;

    // Check for and remove previous requests
    let filter = doc! {
        "user_email": email_txt
    };
    reset_requests.delete_many(filter, None).await?;

    // Store a reset request for the user to get back
    let pword_request = PasswordChangeRequest {
        user_email: String::from(email_txt),
        code
    };

    // Create the new password
    reset_requests.insert_one(pword_request, None).await?;
    Ok(())
}

// Given a code from a reset email, reset a user password
pub async fn change_password(
        email_txt: &str, code: &str, pword_txt: &str,
        db_user: &str, db_pword: &str) -> Result<(), Box<dyn Error>> {
    let db_login = String::from(DB_LOGIN[0]) + db_user + DB_LOGIN[1] + db_pword + DB_LOGIN[2];

    let client = Client::with_uri_str(db_login).await?;
    let vr_db = client.database(DB);
    let reset_requests: Collection<PasswordChangeRequest> = vr_db.collection(PWORD_CHANGE_COLL);

    // Get the request code
    let filter = doc! {
        "user_email": email_txt
    };
    let req = reset_requests.find_one(filter, None).await?;

    // Check for auth errors
    if req.is_none() {
        return Err("No reset request found. Please try again.".into())
    }
    if code != req.unwrap().code {
        return Err("Incorrect code provided.".into())
    }

    // Get new hash and salt
    let salt = get_random_salt();
    let salt_and_pass = salt.clone() + pword_txt;
    let hash = get_hash(&salt_and_pass.to_string());

    // Change password
    let users: Collection<User> = vr_db.collection(USER_COLL);
    let filter = doc! {
        "email": email_txt
    };
    let update = doc! {
        "$set": {
            "pword_salt": salt,
            "pword_hash": hash
        }
    };
    users.update_one(filter, update, None).await?;

    // Delete old request
    let filter = doc! {
        "email": email_txt
    };
    reset_requests.delete_one(filter, None).await?;

    Ok(())
}

/* Helper Functions */

// Make sure an email exists
async fn assert_valid_email(email_txt: &str) -> Result<(), Box<dyn Error>> {
    let request_url = format!("https://isitarealemail.com/api/email/validate?email={}", email_txt);
    let result = get(&request_url).await?.text().await?;

    if result == String::from("{\"status\":\"valid\"}") {
        Ok(())
    } else {
        Err(format!("Email '{}' is invalid.", email_txt))?
    }
}

// Helper function to make sure the email doesn't exist
async fn assert_unused_email(
        email_txt: &str, users: &Collection<User>) -> Result<(), Box<dyn Error>> {
    let filter = doc!{
        "email": email_txt
    };
    let result = users.find_one(filter, None).await?;
    match result {
        Some(_) => Err(format!("Email '{}' already in use.", email_txt).to_owned())?,
        None => Ok(())
    }
}

// Create a random number with a lot of digits for salting
fn get_random_salt() -> String {
    let mut hash_str = String::new();

    let mut rng = thread_rng();
    for _ in 0..SALT_LEN {
        let digit = rng.gen_range(0..10);
        let rand_str = format!("{}", digit);
        hash_str += &rand_str;
    }

    hash_str
}

// Hash a string
fn get_hash(msg: &str) -> String {
    let mut hasher = DefaultHasher::new();
    msg.hash(&mut hasher);
    let num = hasher.finish();
    format!("{:x}", num)
}

// Send an email with a password reset code
async fn send_reset_email(
        user_email: &str, code: &str, email_pword: &str) -> Result<(), Box<dyn Error>> {
    println!("Sending email to {}", user_email);

    println!("Creating email to send to {}.", user_email);
    let email = EmailBuilder::new()
        .to((user_email, "Simple Web Browser User"))
        .from((EMAIL_EMAIL, "Simple Web Browser"))
        .subject("Password Reset Code")
        .text(format!("Code: {}", code))
        .build()
        .unwrap();

    println!("Create mailer.");
    let mut mailer = SmtpClient::new_simple(EMAIL_SMTP)?
        .credentials(Credentials::new(EMAIL_EMAIL.into(), email_pword.into()))
        .transport();

    println!("Send it.");
    mailer.send(email.into())?;

    Ok(())
}

