use ntex::web::{get, Error as WebError, HttpRequest, HttpResponse};
use ntex_files::NamedFile;
use std::{fs::File, io::Read, path::{Path, PathBuf}};

#[get("/")]
pub async fn index() -> Result<HttpResponse, WebError> {
    let mut content = String::new();
    let index_path = Path::new("./index.html");
    let fourofour_path = Path::new("./404.html");

    if index_path.is_file() {
        let mut file = File::open(index_path)?;
        file.read_to_string(&mut content)?;
        return Ok(HttpResponse::Ok().content_type("text/html").body(content));
    }
    
    if fourofour_path.is_file() {
        let mut fourofour_file = File::open(fourofour_path)?;
        fourofour_file.read_to_string(&mut content)?;
        return Ok(HttpResponse::NotFound().content_type("text/html").body(content));
    }
    
    return Ok(HttpResponse::NotFound().content_type("text/html").body("<h1> 404 Not Found <h1>"));    
}

pub async fn files(req: HttpRequest) -> Result<HttpResponse, ntex::web::Error> {
    let path: PathBuf = req.match_info().query("filename").parse()?;
    let file = NamedFile::open(PathBuf::from("./").join(path))?;
    Ok(file.into_response(&req))
}

#[get("/{dir}")]
pub async fn get_from_subdir(req: HttpRequest) -> Result<HttpResponse, ntex::web::Error> {
    let base_dir = PathBuf::from("./");
    let dir: PathBuf = req.match_info().query("dir").parse()?;
    let file_path = base_dir.join(dir).join("index.html");
    let file = NamedFile::open(file_path)?;
    Ok(file.into_response(&req))
}
