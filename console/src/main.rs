use nipah_tokenizer::tokenizer::tokenize;

fn main() {
    let options = nipah_tokenizer::options::default();
    
    let text = "Hello, World!".to_string();

    let tokens = tokenize(text, &options);
    println!("{:#?}", tokens);
}
