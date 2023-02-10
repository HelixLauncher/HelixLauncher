fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = helixlauncher_java::search::search_java()?;
    println!("{result:?}");
    Ok(())
}
