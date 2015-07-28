extern crate handlebars;
extern crate rustc_serialize;
extern crate pulldown_cmark;

use renderer::Renderer;
use book::{BookItems, BookItem, BookConfig};
use theme;

use std::path::{Path, PathBuf, Component};
use std::fs::{self, File, metadata};
use std::error::Error;
use std::io::{self, Read, Write};
use std::collections::BTreeMap;

use self::handlebars::Handlebars;
use self::rustc_serialize::json::{Json, ToJson};

use self::pulldown_cmark::Parser;
use self::pulldown_cmark::html;

pub struct HtmlHandlebars;

impl Renderer for HtmlHandlebars {
    fn render(&self, book: BookItems, config: &BookConfig) -> Result<(), Box<Error>> {

        let mut handlebars = Handlebars::new();

        // Load template
        let t = theme::get_index_hbs();

        // Register template
        try!(handlebars.register_template_string("index", t.to_owned()));

        let mut data = try!(make_data(book.clone(), config));

        // Render a file for every entry in the book
        for (_, item) in book {

            if item.path != PathBuf::new() {

                let path = config.src().join(&item.path);
                println!("Open file: {:?}", path);
                let mut f = try!(File::open(&path));
                let mut content: String = String::new();

                try!(f.read_to_string(&mut content));

                // Render markdown using the pulldown-cmark
                content = render_html(&content);

                // Remove content from previous file and render content for this one
                data.remove("content");
                data.insert("content".to_string(), content.to_json());

                // Rendere the handlebars template with the data
                let rendered = try!(handlebars.render("index", &data));

                // Write to file
                let mut file = try!(create_file(config.dest(), &item.path));
                try!(file.write_all(&rendered.into_bytes()));
            }
        }

        // Copy static files (js, css, images, ...)

        // JavaScript
        let mut js_file = try!(File::create(config.dest().join("book.js")));
        try!(js_file.write_all(theme::get_js()));

        // Css
        let mut css_file = try!(File::create(config.dest().join("book.css")));
        try!(css_file.write_all(theme::get_css()));

        Ok(())
    }
}

impl HtmlHandlebars {
    pub fn new() -> Self {
        HtmlHandlebars
    }

    fn _load_template(&self, path: &Path) -> Result<String, Box<Error>> {
        let mut file = try!(File::open(path));
        let mut s = String::new();
        try!(file.read_to_string(&mut s));
        Ok(s)
    }
}

fn create_file(working_directory: &Path, path: &Path) -> Result<File, Box<Error>> {

    println!("create_file:\n\t{:?}\n\t{:?}", working_directory, path);

    // Extract filename
    let mut file_name;
    if let Some(name) = path.file_stem() {
        file_name = String::from(name.to_str().unwrap());
    }
    else { return Err(Box::new(io::Error::new(io::ErrorKind::Other, "No filename"))) }

    file_name.push_str(".html");

    // Delete filename from path
    let mut path = path.to_path_buf();
    path.pop();

    // Create directories if they do not exist
    let mut constructed_path = PathBuf::from(working_directory);

    for component in path.components() {

        let mut dir;
        match component {
            Component::Normal(_) => { dir = PathBuf::from(component.as_os_str()); },
            _ => continue,
        }

        constructed_path.push(&dir);

        println!("constructed path= {:?}\ndir= {:?}", constructed_path, dir.as_os_str());

        // Check if path exists
        match metadata(&constructed_path) {
            // Any way to combine the Err and first Ok branch ??
            Err(_) => {
                try!(fs::create_dir(&constructed_path))
            },
            Ok(f) => {
                if !f.is_dir() {
                    try!(fs::create_dir(&constructed_path))
                } else {
                    println!("Exists ??");
                    continue
                }
            },
        }

    }

    let file = try!(File::create(
        constructed_path.join(file_name)
    ));

    Ok(file)
}


fn make_data(book: BookItems, config: &BookConfig) -> Result<BTreeMap<String,Json>, Box<Error>> {

    /*
        Function to make the JSon data for the handlebars template:

        {
            "language": ???,
            "chapters": [
                {
                    "section": section,
                    "chapter": BookItem,
                },
                {
                    ...
                },
            ],
        }

    */

    let mut data  = BTreeMap::new();
    data.insert("language".to_string(), "en".to_json());

    let mut chapters = vec![];

    for (section, item) in book {
        let mut chapter = BTreeMap::new();
        chapter.insert("section".to_string(), section.to_json());
        chapter.insert("chapter".to_string(), item.to_json());

        chapters.push(chapter);
    }

    data.insert("chapters".to_string(), chapters.to_json());

    Ok(data)
}

fn render_html(text: &str) -> String {
    let mut s = String::with_capacity(text.len() * 3 / 2);
    let p = Parser::new(&text);
    html::push_html(&mut s, p);
    s
}
