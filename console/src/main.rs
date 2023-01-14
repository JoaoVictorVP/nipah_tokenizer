use nipah_tokenizer::tokenizer::tokenize;

fn main() {
    let options = nipah_tokenizer::options::default();
    
    let text = "My money is: + 1000.0f".to_string();

    let tokens = tokenize(text, &options);
    println!("{:#?}", tokens);
}
