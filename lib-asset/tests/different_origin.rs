use lib_asset::{AssetRoot, FsResolver};

#[test]
fn test_origin() {
    let mut resolver = FsResolver::new();
    resolver.set_root(AssetRoot::Default, "../assets");
    let quaver_path = resolver.get_path(AssetRoot::Default, "quaver.ttf");
    assert!(std::fs::exists(quaver_path).unwrap());
}
