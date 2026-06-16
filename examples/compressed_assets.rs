use scenix::{ASSET_FORMATS, AssetFormatSupport, support_for_extension};

fn main() {
    for info in ASSET_FORMATS {
        println!("{}: {:?}", info.name, info.support);
    }

    let ktx2 = support_for_extension("ktx2").expect("ktx2 row");
    assert_eq!(ktx2.support, AssetFormatSupport::Partial);
}
