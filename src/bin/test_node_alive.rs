extern crate reqwest;

fn main() {
    let mut response = reqwest::get("https://10.0.1.1/api/v1/node-info").unwrap();
    assert_eq!(200, response.status(), "Invalid HTTP code");
    assert_eq!("", response.text().unwrap());
}
