use winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icons/elmer.ico"); // Path to your .ico file
        res.compile().unwrap();
    }
}
