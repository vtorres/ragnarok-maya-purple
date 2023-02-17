use winres::WindowsResource;

fn main() {
    let mut res = WindowsResource::new();

    res.set_icon("./assets/baphomet.ico")
        .set("InternalName", "Ragnarok - Maya Purple")
        .set_language(0x0409)
        .set("CompanyName", "https://github.com/vtorres/");

    res.compile().unwrap();
}