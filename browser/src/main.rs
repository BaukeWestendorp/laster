fn main() {
    let html = r#"
        <html>
            <head>
                <title>Test</title>
            </head>
            <body>
                <h1>Hello, world!</h1>
            </body>
        </html>
    "#;
    let dom = dom::Dom::parse(html);
    dbg!(dom);
}
