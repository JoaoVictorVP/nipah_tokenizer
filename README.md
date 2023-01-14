# nipah_tokenizer
A powerful yet simple text tokenizer for your everyday needs!

![322977877_dark_magic_and_black_fire-transformed](https://user-images.githubusercontent.com/98046863/212478423-b91038a5-5149-4534-b80a-641dea5355cb.png)

## How To Use
1. Obtain the default options (or create yourself your configuration):
```rust
let options = nipah_tokenizer::options::default();
```
2. Get some text
```rust
let text = "Hello, World!".to_string();
```
3. Tokenize your text!
```rust
use nipah_tokenizer::tokenizer::tokenize;

let tokens = tokenize(text, &options);
```

Output will be like:
```
["Hello": Id] [Comma] ["World": Id] [Exclamation]
```

You can use it as your hearth desires, and it is pretty versatile

Happy coding!
