fn main() {
    embed_resource::compile("src/helixlauncher.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
    cc::Build::new().file("src/exports.c").compile("exports");
}
