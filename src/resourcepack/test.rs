use std::str::FromStr;

use super::{BlockStateResource, MinecraftResourceIdentifier, ParseError, blockstate::BlockState};

#[test]
fn parse_resource_identifiers() -> Result<(), ParseError> {
    assert_eq!(
        MinecraftResourceIdentifier::from_str("dirt")?.to_string(),
        "minecraft:dirt"
    );
    assert_eq!(
        MinecraftResourceIdentifier::from_str("fancy_mod31:coolblaster.notbullet")?.to_string(),
        "fancy_mod31:coolblaster.notbullet"
    );
    assert_eq!(
        MinecraftResourceIdentifier::from_str("notcool/mod:great_item").unwrap_err(),
        ParseError::InvalidCharactersInNamespace
    );
    assert_eq!(
        MinecraftResourceIdentifier::from_str("coolmod2:bad,VERYBAD.item").unwrap_err(),
        ParseError::InvalidCharactersInPath
    );
    assert_eq!(
        MinecraftResourceIdentifier::from_str("").unwrap_err(),
        ParseError::EmptyString
    );

    Ok(())
}

fn blockstate_equalit() {
    let mut blockstate_1 = BlockState::new();
    blockstate_1.insert("flying", "true");
    blockstate_1.insert("lines", "3");

    let mut blockstate_2 = BlockState::new();
    blockstate_1.insert("lines", "3");
    blockstate_1.insert("flying", "true");

    assert_eq!(blockstate_1, blockstate_2);
}
