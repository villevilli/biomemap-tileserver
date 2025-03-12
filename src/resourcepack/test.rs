use std::str::FromStr;

use super::{MinecraftResourceIdentifier, ParseError};

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
