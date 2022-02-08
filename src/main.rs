use std::env;
use select::document::{Document};
use select::predicate::{Name, Class, And, Descendant, Attr};
use regex::Regex;
use std::io::BufWriter;
use std::io::Write;

const DEST_DIR: &'static str = ".";

async fn fetch_page(url: &str) -> Result<Document, Box<dyn std::error::Error>> {
    let text: String = reqwest::get(url).await?.text_with_charset("gbk").await?;
    let doc = Document::from(text.as_ref());
    Ok(doc)
}

async fn download_novel(url: &str) -> Result<(), Box<dyn std::error::Error>>{
    let doc: Document = fetch_page(url).await.expect("Page not exist");
    let index_url: &str = doc.find(Name("a")).filter_map(|item| {
        let text = item.text();
        if text == "小说目录" {
            Some(item.attr("href").expect("Invalid Page"))
        }else{
            None
        }
    }).next().expect("Index button not found.");
    let doc: Document = fetch_page(index_url).await.expect("Invalid Page");
    let title: String = doc.find(Attr("id", "title")).next().unwrap().text();
    let chapter_tail_iter = doc.find(
            Descendant(And(Name("td"), Class("ccss")), Name("a")))
            .map(|item| item.attr("href").expect("Invalid Index Page"));
    let target_fn = format!("{}.md", title);
    let mut target_f: std::fs::File = std::fs::OpenOptions::new().create(true).write(true).append(true).open(target_fn)?;
    let mut writer = BufWriter::new(target_f);
    for ct in chapter_tail_iter {
        let chapter_url = index_url.replace("index.htm", ct);
        let chapt_doc = fetch_page(&chapter_url).await?;
        let chapt_title: String = chapt_doc.find(Attr("id", "title")).next().unwrap().text();
        let content = chapt_doc.find(Attr("id", "content")).next().unwrap().text().replace("  ", "");
        writer.write_fmt(format_args!("\n# {}\n\n", chapt_title))?;
        writer.write(content.as_bytes())?;
        writer.write("\n\n".as_bytes())?;
        writer.flush()?;
        println!("  Chapter: {}", chapt_title);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for url in env::args().skip(1) {
        let res: Result<(), Box<dyn std::error::Error>> = download_novel(&url).await;
        match res {
            Ok(()) => {
                println!("Done: {}", url)
            }
            Err(err) => {
                println!("Error: {}", url);
                println!("{}", err)
            }
        }
    }
    // let resp = reqwest::get("")
    //     .await?

    //     .await?;
    // println!("{:#?}", resp);
    Ok(())
}
