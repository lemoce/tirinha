extern crate reqwest;
extern crate sdl2;
extern crate select;
extern crate tempfile;

use std::path::Path;
use std::string::String;
use std::result::Result;

use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name};

use tempfile::NamedTempFile;

use sdl2::image::{LoadTexture, InitFlag};
use sdl2::render::WindowCanvas;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;

fn draw_image(path: &Path, canvas: &mut WindowCanvas) -> Result<(), String> {
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture(path)?;

    canvas.copy(&texture, None, None)?;
    canvas.present();

    Ok(())
}

fn run(image_files: &Vec<&NamedTempFile>) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem.window("Tirinha", 1150, 370)
      .position_centered()
      .build()
      .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().software().build().map_err(|e| e.to_string())?;

    let mut idx = 0;
    
    draw_image(image_files[idx].path(), &mut canvas)?;
    
    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit{..} |
                Event::KeyDown {keycode: Option::Some(Keycode::Escape), ..} =>
                    break 'mainloop,
                Event::KeyDown {keycode: Option::Some(Keycode::Right), ..} => {
                    if idx < 4 {
                        idx = idx + 1;
                        draw_image(image_files[idx].path(), &mut canvas)?;
                    }
                },
                Event::KeyDown {keycode: Option::Some(Keycode::Left), ..} => {
                    if idx > 0 {
                        idx = idx - 1;
                        draw_image(image_files[idx].path(), &mut canvas)?;
                    }
                },
                Event::Window{ win_event: WindowEvent::Exposed, .. } => 
                    draw_image(image_files[idx].path(), &mut canvas)?,
                _ => { }
            }
        }
    }

    Ok(())
}

fn get_response_body() -> Result<String, reqwest::Error> {
    let content = reqwest::get("https://cultura.estadao.com.br/quadrinhos")?.text()?;

    Ok(content)
}

fn extract_image_urls(html_body: String) -> Vec<String> {

    let document = Document::from(html_body.as_str());

    let mut result = vec![];

    let nodes: Vec<Node> = document
        .find(Class("quadrinho-wrapper"))
        .collect();
    
    for node in nodes
    {
        let anchor = node.find(Name("img")).next().unwrap().attr("data-src-desktop").unwrap();
        result.push(String::from(anchor));
    }    

    result
}

fn main() {

    let image_urls = extract_image_urls(get_response_body().unwrap());

    let mut file_vec: Vec<NamedTempFile> = vec![];

    for _ in 0 .. image_urls.len() {
        let file = NamedTempFile::new().unwrap();
        file_vec.push(file);
    }

    let zip_urls_files = image_urls.iter().zip(file_vec.iter());

    let downloaded_files: Vec<&NamedTempFile> = zip_urls_files.map(|(url, mut f)| {
        let mut resp = reqwest::get(url.as_str()).unwrap();
        resp.copy_to(&mut f).unwrap();
        f
    }).collect();

    run(&downloaded_files).unwrap();
    
}