use crate::domain::general::ReplaceParams;
use tokio::fs;

pub async fn start_message(location: String) {
    //read the logo.txt file, then print the logo
    let mut param_vec = Vec::new();
    let param = ReplaceParams {
        old_str: "[VERSION]".to_string(),
        new_str: env!("CARGO_PKG_VERSION").parse().unwrap(),
    };
    param_vec.push(param);

    match fs::read_to_string("src/utils/logo.txt").await {
        Ok(content) => {
            let content = replace_bulk(content, &param_vec).await;
            print!("{}", content);
        }
        Err(e) => {
            eprintln!("Failed to read logo.txt: {}", e);
        }
    }
    println!();
    println!("Server listening on {}.", location)
}

async fn replace_bulk(text: String, params: &[ReplaceParams]) -> String {
    let mut text_result = text;
    for param in params {
        text_result = text_result.replace(&param.old_str, &param.new_str);
    }
    text_result
}
