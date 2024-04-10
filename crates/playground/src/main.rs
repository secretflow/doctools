fn main() -> anyhow::Result<()> {
  println!(
    "{:?}",
    url::Url::from_directory_path(std::env::current_dir()?)
      .map_err(|_| anyhow::anyhow!("???"))?
      .join("src\\main.rs/")?
      .to_file_path()
  );
  Ok(())
}
