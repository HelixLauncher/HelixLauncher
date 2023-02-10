fn main() {
    embed_resource::compile("src/helixlauncher.rc");
    cc::Build::new().file("src/exports.c").compile("exports");
}
