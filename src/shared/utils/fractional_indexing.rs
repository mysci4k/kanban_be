use fractional_index::FractionalIndex;
use serde::Deserialize;

pub struct FractionalIndexGenerator;

impl FractionalIndexGenerator {
    pub fn first() -> String {
        FractionalIndex::default().to_string()
    }

    pub fn after(position: &str) -> Result<String, String> {
        let current = Self::from_string(position)?;

        Ok(FractionalIndex::new_after(&current).to_string())
    }

    pub fn before(position: &str) -> Result<String, String> {
        let current = Self::from_string(position)?;

        Ok(FractionalIndex::new_before(&current).to_string())
    }

    pub fn between(position1: &str, position2: &str) -> Result<String, String> {
        let first = Self::from_string(position1)?;
        let second = Self::from_string(position2)?;

        FractionalIndex::new_between(&first, &second)
            .map(|idx| idx.to_string())
            .ok_or_else(|| {
                "Cannot create index between positions - they may be adjacent".to_string()
            })
    }

    pub fn generate_for_position(
        existing_positions: &[String],
        target_index: usize,
    ) -> Result<String, String> {
        if existing_positions.is_empty() {
            return Ok(Self::first());
        }

        let len = existing_positions.len();

        if target_index == 0 {
            Self::before(&existing_positions[0])
        } else if target_index >= len {
            Self::after(&existing_positions[len - 1])
        } else {
            Self::between(
                &existing_positions[target_index - 1],
                &existing_positions[target_index],
            )
        }
    }

    fn from_string(hex_str: &str) -> Result<FractionalIndex, String> {
        #[derive(Debug, Deserialize)]
        struct Wrapper {
            #[serde(with = "fractional_index::stringify")]
            value: FractionalIndex,
        }

        let json = format!(r#"{{"value":"{}"}}"#, hex_str);

        let wrapper: Wrapper = serde_json::from_str(&json)
            .map_err(|err| format!("Invalid fractional index '{}': {}", hex_str, err))?;

        Ok(wrapper.value)
    }
}
