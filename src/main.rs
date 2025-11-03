mod graph;
mod mql;

use mql::lexer::{Lexer, Token, Span};
use mql::parser::Parser;

fn main() {
    // let create_statement = Lexer::new("create author:jk { name = \"J.K. Rowling\" };").collect::<Vec<(Token, Span)>>();
    // let create_with_link_statement = Lexer::new(r#"
    // create book:hp1 {
    //     title = "Philosopher’s Stone",
    //     pages = 223,
    //     author -> author:jk
    // };"#).collect::<Vec<(Token, Span)>>();
    // let another_create_statement = Lexer::new("create book:hp2 { title = \"Chamber of Secrets\" };").collect::<Vec<(Token, Span)>>();
    // let link_statement = Lexer::new("link author:jk -> book:hp2;").collect::<Vec<(Token, Span)>>();
    // let select_statement = Lexer::new("select * from author where name = \"J.K. Rowling\";"); //.collect::<Vec<(Token, Span)>>();
    // let select_with_id_statement = Lexer::new("select title from book:hp1;"); // .collect::<Vec<(Token, Span)>>();
    let select_with_relation_statement = Lexer::new("select * from author->book where title like \"Harry Potter\";"); //.collect::<Vec<(Token, Span)>>();
    //
    // println!("{create_statement:#?}");
    // println!("{create_with_link_statement:#?}");
    // println!("{another_create_statement:#?}");
    // println!("{link_statement:#?}");
    // println!("{select_statement:#?}");
    // println!("{select_with_id_statement:#?}");
    // println!("{select_with_relation_statement:#?}");
    let mut parser = Parser::new(select_with_relation_statement);
    let ast = parser.parse();

    println!("{:?}", ast);
}
