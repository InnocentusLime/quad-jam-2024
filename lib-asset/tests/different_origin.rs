use lib_asset::FsResolver;

#[test]
fn test_origin() {
    let mut resolver = FsResolver::new();
    resolver.set_assets_dir("../assets").unwrap();
    let quaver_path = resolver.asset_path("quaver.ttf");
    assert!(std::fs::exists(quaver_path).unwrap());
}
